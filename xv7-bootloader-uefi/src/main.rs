#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(asm)]
#![feature(box_patterns)]
#![feature(box_syntax)]

mod config;
mod io;
mod loader;
mod mem;
mod paging;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use chrono::prelude::*;
use uefi::prelude::*;

use config::*;

use bootinfo::{KernelArgs, KernelEntryFn, KERNEL_ARGS_MAGIC};

static mut KERNEL_ENTRY: usize = 0x0;

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize UEFI environment");
    let _ = system_table.stdout().clear().unwrap();

    let boot_services = system_table.boot_services();

    boot_services
        .set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");

    print_system_information(&system_table).expect_success("Failed to print system information");

    // load kernel ELF image.
    let entry_offset = loader::load_elf(boot_services, KERNEL_IMAGE_PATH);

    info!(
        "Kernel entry point: {:#x} + {:#x}",
        KERNEL_VIRTUAL_BASE, entry_offset
    );

    // Exit boot services and jump to the kernel.
    info!("Exiting UEFI boot services and jumping to the kernel");
    let mmap_size = boot_services.memory_map_size();
    let mut mmap_buf = vec![0u8; mmap_size];
    system_table
        .exit_boot_services(image_handle, &mut mmap_buf)
        .expect_success("UEFI exit boot services failed");

    unsafe {
        paging::paging();
    }

    unsafe {
        KERNEL_ENTRY = KERNEL_VIRTUAL_BASE + entry_offset;
        asm!("mov $0, %rsp" : : "r"(STACK_VIRTUAL + STACK_SIZE) : "memory" : "volatile");
        // NOTICE: after we changed rsp, all local variables are no longer avaliable
        // and we must call another function immediately
        call_kernel_entry();
    }
}

/// This function runs on new kernel stack.
unsafe fn call_kernel_entry() -> ! {
    use core::mem;
    let kernel_entry: KernelEntryFn = mem::transmute(KERNEL_ENTRY);
    let args = KernelArgs {
        magic: KERNEL_ARGS_MAGIC,
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

    info!("\nSystem Information:\n");

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
            let smbios = unsafe { *(addr as *const bootinfo::SMBIOSEntryPoint) };
            info!("{:?}", smbios);
        }
    }

    let boot_services = system_table.boot_services();

    let gop = boot_services
        .locate_protocol::<uefi::proto::console::gop::GraphicsOutput>()
        .expect_success("");

    let gop = unsafe { &mut *gop.get() };

    let mut buf = gop.frame_buffer();

    info!("Graphic buffer: {:p}, {:#x}", buf.as_mut_ptr(), buf.size());

    Ok(().into())
}
