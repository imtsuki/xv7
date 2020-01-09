#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use chrono::prelude::*;
use uefi::prelude::*;

#[no_mangle]
extern "C" fn __rust_probestack() {}

#[entry]
fn efi_main(_image: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize utilities");
    let _ = system_table.stdout().clear().unwrap();

    info!("Hello, UEFI");

    let now = system_table.runtime_services().get_time().unwrap().unwrap();
    let now = Utc
        .ymd(now.year() as i32, now.month() as u32, now.day() as u32)
        .and_hms(now.hour() as u32, now.minute() as u32, now.second() as u32);
    info!("TimeZone Bupt/Jwxt: {}", now);

    let now = now.with_timezone(&FixedOffset::east(8 * 3600));
    info!("TimeZone Asia/Shanghai: {}", now);

    let boot_services = system_table.boot_services();
    let map_size = boot_services.memory_map_size();
    info!("map_size: {}", map_size);

    use x86_64::registers::control::Cr3;
    let (page_table, _) = Cr3::read();
    info!(
        "Current level 4 page table is located at: {:?}",
        page_table.start_address()
    );

    loop {}
}
