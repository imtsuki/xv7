#![no_std]
#![no_main]
#![cfg_attr(doc, allow(unused_attributes))]
#![feature(abi_efiapi)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(llvm_asm)]
#![feature(maybe_uninit_extra)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
mod macros;

mod config;
mod io;
mod loader;
mod paging;

use alloc::boxed::Box;
use core::mem::MaybeUninit;

use boot::MemoryMapIter;
use boot::BOOT_ARGS_MAGIC;
use boot::{BootArgs, FrameBufferDescriptor, KernelEntry, KernelEntryFn, MemoryMap};

use chrono::prelude::*;
use uefi::prelude::*;
use x86_64::{
    structures::paging::{PageSize, Size4KiB},
    PhysAddr, VirtAddr,
};

use config::*;

static mut KERNEL_ENTRY: KernelEntry = KernelEntry(VirtAddr::new_truncate(0x0));
static mut FRAME_BUFFER_BASE: u64 = 0x0;
static mut FRAME_BUFFER_LEN: usize = 0x0;
static mut RESOLUTION: (usize, usize) = (0, 0);
static mut MMAP_ITER: MaybeUninit<MemoryMapIter> = MaybeUninit::uninit();

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize UEFI environment");
    let _ = system_table.stdout().clear().unwrap();

    let boot_services = system_table.boot_services();

    boot_services
        .set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");

    print_system_information(&system_table).expect_success("Failed to print system information");

    // Initialize our "kernel" frame allocator which marks frames as `MEMORY_TYPE_KERNEL`.
    let mut frame_allocator = paging::KernelFrameAllocator::new(boot_services);

    let mut page_table = paging::init_recursive(&mut frame_allocator);
    // load kernel ELF image.
    let kernel_entry = loader::load_elf(
        boot_services,
        &mut page_table,
        &mut frame_allocator,
        dbg!(KERNEL_IMAGE_PATH),
    );

    dbg!(kernel_entry);

    let mmap_size = boot_services.memory_map_size();
    let mut mmap_buf = vec![0u8; mmap_size * 2];
    let (_, mmap_iter) = boot_services
        .memory_map(&mut mmap_buf)
        .expect_success("Failed to get memory map");

    let max_addr = PhysAddr::new(
        mmap_iter
            .map(|m| m.phys_start + m.page_count * Size4KiB::SIZE - 1)
            .max()
            .unwrap()
            .max(0xFFFF_FFFF),
    );

    // Map complete pyhsical memory to `PAGE_OFFSET_BASE`.
    paging::map_physical_memory(
        VirtAddr::new(dbg!(PAGE_OFFSET_BASE)),
        max_addr,
        &mut page_table,
        &mut frame_allocator,
    );

    paging::map_stack(
        VirtAddr::new(KERNEL_STACK_TOP),
        KERNEL_STACK_SIZE,
        &mut page_table,
        &mut frame_allocator,
    );

    // Exit boot services and jump to the kernel.
    info!("Exiting UEFI boot services and jumping to the kernel");
    let mmap_size = boot_services.memory_map_size();
    let mmap_buf = Box::leak(vec![0u8; mmap_size * 2].into_boxed_slice());
    let (_, mmap_iter) = system_table
        .exit_boot_services(image_handle, mmap_buf)
        .expect_success("UEFI exit boot services failed");

    unsafe {
        // FIXME: A dirty HACK to get `mmap_iter` to point to mapped memory.
        let mut tuple: (u64, usize, usize, usize, usize) = core::mem::transmute(mmap_iter);
        tuple.0 += PAGE_OFFSET_BASE;
        let mmap_iter: MemoryMapIter = core::mem::transmute(tuple);
        MMAP_ITER.write(mmap_iter);

        KERNEL_ENTRY = kernel_entry;
        llvm_asm!("mov $0, %rsp" : : "r"(KERNEL_STACK_TOP) : "memory" : "volatile");
        // NOTICE: after we changed rsp, all local variables are no longer avaliable
        // and we must call another function immediately
        call_kernel_entry();
    }
}

/// This function runs on new kernel stack.
unsafe fn call_kernel_entry() -> ! {
    use core::mem;
    let kernel_entry: KernelEntryFn = mem::transmute(KERNEL_ENTRY);
    let args = BootArgs {
        magic: BOOT_ARGS_MAGIC,
        frame_buffer: FrameBufferDescriptor {
            base: PhysAddr::new(FRAME_BUFFER_BASE),
            len: FRAME_BUFFER_LEN,
            resolution: RESOLUTION,
        },
        memory_map: MemoryMap {
            iter: MMAP_ITER.assume_init_read(),
        },
    };
    kernel_entry(&args);
}

fn print_system_information(system_table: &SystemTable<Boot>) -> uefi::Result {
    info!(
        "{} v{}",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION")
    );
    info!("By {}", env!("CARGO_PKG_AUTHORS"));

    info!(
        "UEFI Firmware {} {:#?}",
        system_table.firmware_vendor(),
        system_table.firmware_revision()
    );

    let now = system_table.runtime_services().get_time().log_warning()?;
    let now = Utc
        .ymd(now.year() as i32, now.month() as u32, now.day() as u32)
        .and_hms(now.hour() as u32, now.minute() as u32, now.second() as u32);
    info!("TimeZone Bupt/Jwxt: {}", now);

    let now = now.with_timezone(&FixedOffset::east(8 * 3600));
    info!("TimeZone Asia/Shanghai: {}", now);

    for e in system_table.config_table() {
        if e.guid == uefi::table::cfg::SMBIOS_GUID {
            let addr = e.address;
            let smbios = unsafe { *(addr as *const boot::SmbiosEntryPoint) };
            debug!("{:?}", smbios);
        }
    }

    let boot_services = system_table.boot_services();

    let gop = boot_services
        .locate_protocol::<uefi::proto::console::gop::GraphicsOutput>()
        .expect_success("");

    let gop = unsafe { &mut *gop.get() };

    let mut buf = gop.frame_buffer();

    info!("Graphic buffer: {:p}, {:#x}", buf.as_mut_ptr(), buf.size());

    unsafe {
        FRAME_BUFFER_BASE = buf.as_mut_ptr() as u64;
        FRAME_BUFFER_LEN = buf.size();
        RESOLUTION = gop.current_mode_info().resolution();
    }

    Ok(().into())
}
