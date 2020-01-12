#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(box_patterns)]
#![feature(box_syntax)]

mod io;
mod mem;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::boxed::Box;
use chrono::prelude::*;
use uefi::prelude::*;

#[no_mangle]
extern "C" fn __rust_probestack() {}

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize UEFI environment");
    let _ = system_table.stdout().clear().unwrap();

    info!(
        "{} v{}",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION")
    );

    info!("By {}", env!("CARGO_PKG_AUTHORS"));

    info!("\nSystem Information:\n");

    info!("Firmware Revision: {:#?}", system_table.firmware_revision());

    let now = system_table.runtime_services().get_time().unwrap().unwrap();
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
            info!("{:#?}", smbios);
        }
    }

    let boot_services = system_table.boot_services();

    boot_services
        .set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");

    use x86_64::registers::control::Cr3;
    let (page_table, _) = Cr3::read();
    info!(
        "Current level 4 page table is located at: {:?}",
        page_table.start_address()
    );

    info!(r"Loading kernel image from \EFI\xv7\kernel");
    let (len, kernel_image) = io::read_file(boot_services, r"\EFI\xv7\kernel")
        .expect_success("Could not load kernel image");

    info!("Kernel image size = {}", len);

    let kernel_elf = xmas_elf::ElfFile::new(&kernel_image).expect("Error format of kernel image");

    info!("Kernel image info:");
    info!("{}", kernel_elf.header);
    let entry_offset = kernel_elf.header.pt2.entry_point();
    let base_address = Box::leak(kernel_image.into_boxed_slice()).as_mut_ptr();

    info!(
        "Kernel entry point found: {:p} + {:#x}",
        base_address, entry_offset
    );

    // Exit boot services and jump to the kernel.
    info!("Exiting UEFI boot services and jumping to the kernel");
    let mmap_size = boot_services.memory_map_size();
    let mut mmap_buf = vec![0u8; mmap_size];
    system_table
        .exit_boot_services(image_handle, &mut mmap_buf)
        .expect_success("UEFI exit boot services failed");

    // No need to relocate our kernel because it is linked as a PIE executable.
    let kernel_entry_ptr = unsafe { base_address.offset(entry_offset as isize) as *const () };
    let kernel_entry: extern "C" fn() = unsafe { core::mem::transmute(kernel_entry_ptr) };

    kernel_entry();

    unreachable!();
}
