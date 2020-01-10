#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(asm)]
#![feature(box_patterns)]
#![feature(box_syntax)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::vec::Vec;
use chrono::prelude::*;
use uefi::prelude::*;
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::Result;

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
    let (len, data) =
        read_file(boot_services, r"\EFI\xv7\kernel").expect_success("Could not load kernel image");

    info!("Kernel image size = {}", len);
    info!(
        "Check kernel image magic number: {}{}{}",
        data[1] as char, data[2] as char, data[3] as char
    );

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn read_file(service: &BootServices, path: &str) -> Result<(usize, Vec<u8>)> {
    let fatfs = service
        .locate_protocol::<SimpleFileSystem>()
        .log_warning()?;
    let fatfs = unsafe { &mut *fatfs.get() };

    let mut volume = fatfs.open_volume().log_warning()?;

    let file_handle = volume
        .open(path, FileMode::Read, FileAttribute::empty())
        .log_warning()?;

    let mut file = match file_handle.into_type().log_warning()? {
        FileType::Regular(regular) => regular,
        FileType::Dir(_) => unreachable!(),
    };

    // Use an empty buffer to retrieve the actual FileInfo size
    let mut empty_buf = Vec::new();
    let len = match *file
        .get_info::<FileInfo>(&mut empty_buf)
        .expect_error("passing an empty buffer will return the size of FileInfo")
        .data()
    {
        Some(len) => len,
        None => unreachable!(),
    };

    let mut file_info = vec![0u8; len];
    let file_info = file
        .get_info::<FileInfo>(&mut file_info)
        .discard_errdata()
        .log_warning()?;

    let mut buf = vec![0u8; file_info.file_size() as usize];
    let len = file.read(&mut buf).discard_errdata().log_warning()?;

    Ok((len, buf).into())
}
