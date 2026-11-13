#![no_std]
#![no_main]
#![cfg_attr(doc, allow(unused_attributes))]
#![feature(sync_unsafe_cell)]

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

use core::cell::SyncUnsafeCell;
use core::mem::MaybeUninit;

use boot::BOOT_ARGS_MAGIC;
use boot::MemoryMapIter;
use boot::{BootArgs, FrameBufferDescriptor, MemoryMap};

use chrono::prelude::*;
use uefi::boot::MemoryType;
use uefi::mem::memory_map::MemoryMap as _;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::cfg::ConfigTableEntry;
use uefi::{Status, entry};
use x86_64::{PhysAddr, VirtAddr};

use config::*;

static BOOT_ARGS: SyncUnsafeCell<MaybeUninit<BootArgs>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().expect("Failed to initialize UEFI environment");
    uefi::system::with_stdout(|stdout| stdout.clear()).unwrap();

    let frame_buffer = print_system_information().expect("Failed to print system information");

    uefi::boot::set_watchdog_timer(0, 0x10000, None).expect("Could not set watchdog timer");

    let mut page_table = paging::BootPageTable::new();
    let kernel_entry = loader::load_elf(&mut page_table, dbg!(KERNEL_IMAGE_PATH));
    dbg!(kernel_entry);

    let mmap = uefi::boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");
    let fb_end = frame_buffer
        .base
        .as_u64()
        .saturating_add(frame_buffer.len as u64)
        .saturating_sub(1);
    let max_addr = PhysAddr::new(
        mmap.entries()
            .filter(|m| m.ty != MemoryType::RESERVED)
            .map(|m| m.phys_start + m.page_count * paging::PAGE_SIZE_4K - 1)
            .max()
            .unwrap_or(0)
            .max(fb_end)
            .max(0xFFFF_FFFF),
    );

    // Map physical memory to `PAGE_OFFSET_BASE` and allocate initial kernel stack.
    page_table.map_physical_memory(VirtAddr::new(dbg!(PAGE_OFFSET_BASE)), max_addr);
    page_table.map_stack(VirtAddr::new(KERNEL_STACK_TOP), KERNEL_STACK_SIZE);

    // Exit boot services, switch to the newly constructed page table, and jump to the kernel.
    info!("Exiting UEFI boot services and jumping to the kernel");
    let mmap = unsafe { uefi::boot::exit_boot_services(Some(MemoryType::LOADER_DATA)) };

    unsafe {
        page_table.activate();

        let raw_buf = mmap.buffer();
        let meta = mmap.meta();
        let mapped_buf = core::slice::from_raw_parts(
            (raw_buf.as_ptr() as u64 + PAGE_OFFSET_BASE) as *const u8,
            raw_buf.len(),
        );
        let mmap_iter = MemoryMapIter::new(mapped_buf, meta.desc_size, mmap.len());

        (*BOOT_ARGS.get()).write(BootArgs {
            magic: BOOT_ARGS_MAGIC,
            frame_buffer,
            memory_map: MemoryMap { iter: mmap_iter },
        });

        let args_ptr = (BOOT_ARGS.get() as u64 + PAGE_OFFSET_BASE) as *const BootArgs;
        core::arch::asm!(
            "mov rsp, {stack}",
            "call {entry}",
            stack = in(reg) KERNEL_STACK_TOP,
            entry = in(reg) kernel_entry.0.as_u64(),
            in("rdi") args_ptr,
            options(noreturn),
        );
    }
}

fn print_system_information() -> uefi::Result<FrameBufferDescriptor> {
    info!(
        "{} v{}",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION")
    );
    info!("By {}", env!("CARGO_PKG_AUTHORS"));

    info!(
        "UEFI Firmware {} {:#?}",
        uefi::system::firmware_vendor(),
        uefi::system::firmware_revision()
    );

    let now = uefi::runtime::get_time()?;
    let now = Utc
        .with_ymd_and_hms(
            now.year() as i32,
            now.month() as u32,
            now.day() as u32,
            now.hour() as u32,
            now.minute() as u32,
            now.second() as u32,
        )
        .unwrap();
    info!("TimeZone Bupt/Jwxt: {}", now);

    let now = now.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap());
    info!("TimeZone Asia/Shanghai: {}", now);

    uefi::system::with_config_table(|config_table| {
        for e in config_table {
            if e.guid == ConfigTableEntry::SMBIOS_GUID {
                let addr = e.address;
                let smbios = unsafe { *(addr as *const boot::SmbiosEntryPoint) };
                debug!("{:?}", smbios);
            }
        }
    });

    let gop_handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = uefi::boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let resolution = gop.current_mode_info().resolution();
    let mut buf = gop.frame_buffer();

    info!("Graphic buffer: {:p}, {:#x}", buf.as_mut_ptr(), buf.size());

    Ok(FrameBufferDescriptor {
        base: PhysAddr::new(buf.as_mut_ptr() as u64),
        len: buf.size(),
        resolution,
    })
}
