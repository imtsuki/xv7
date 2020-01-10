#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(asm)]
#![feature(box_patterns)]
#![feature(box_syntax)]

mod boot_info;
mod io;
mod mem;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::string::String;
use alloc::vec::Vec;
use chrono::prelude::*;
use uefi::prelude::*;

#[no_mangle]
extern "C" fn __rust_probestack() {}

#[entry]
fn efi_main(_image: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize UEFI environment");
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

    boot_services
        .set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");

    let map_size = boot_services.memory_map_size();
    info!("map_size: {}", map_size);

    use x86_64::registers::control::Cr3;
    let (page_table, _) = Cr3::read();
    info!(
        "Current level 4 page table is located at: {:?}",
        page_table.start_address()
    );

    info!(r"Load kernel image from \EFI\xv7\kernel");
    let (len, data) = io::read_file(boot_services, r"\EFI\xv7\kernel")
        .expect_success("Could not load kernel image");

    info!("Kernel image size = {}", len);

    deal_with_elf(data).expect("ELF processing failed");

    mem::memory_map(boot_services).expect_success("memory failed");

    for e in system_table.config_table() {
        if e.guid == uefi::table::cfg::ACPI2_GUID {
            let desc = unsafe { *(e.address as *const RSDPDescriptor20) };
            info!("{:?}", desc);
            info!(
                "{:?} {:?}",
                String::from_utf8(desc.first_part.signature.to_vec()),
                String::from_utf8(desc.first_part.oem_id.to_vec()),
            );
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct RSDPDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct RSDPDescriptor20 {
    first_part: RSDPDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

fn deal_with_elf(raw: Vec<u8>) -> core::result::Result<(), &'static str> {
    let elf = xmas_elf::ElfFile::new(&raw)?;
    info!("Kernel image info:");
    info!("{}", elf.header);
    Ok(())
}
