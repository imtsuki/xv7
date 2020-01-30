#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(asm)]
#![feature(box_patterns)]
#![feature(box_syntax)]

mod io;
mod mem;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

/*
#[cfg(target_pointer_width = "64")]
use goblin::elf64 as elf;

#[cfg(target_pointer_width = "32")]
use goblin::elf32 as elf;
*/

use chrono::prelude::*;
use goblin::elf;
use uefi::prelude::*;
use x86_64::registers::control::Cr3;
use zeroize::Zeroize;

const KERNEL_IMAGE_PATH: &'static str = r"\EFI\xv7\kernel";

#[allow(unused)]
const VIRTUAL_OFFSET: usize = 0xFFFF800000000000;

const KERNEL_PHYSICAL_BASE: usize = 0x100000;

#[allow(unused)]
const STACK_PHYSICAL: usize = 0x80000;

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect_success("Failed to initialize UEFI environment");
    let _ = system_table.stdout().clear().unwrap();

    print_system_information(&system_table).expect_success("Failed to print system information");

    let boot_services = system_table.boot_services();

    boot_services
        .set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");

    info!(r"Loading kernel image from {}", KERNEL_IMAGE_PATH);
    let (len, kernel_image) = io::read_file(boot_services, KERNEL_IMAGE_PATH)
        .expect_success("Could not load kernel image");

    info!("Kernel image size = {}", len);

    let kernel_elf = elf::Elf::parse(&kernel_image).expect("Failed to parse ELF file");

    info!(
        "Now loading kernel to KERNEL_PHYSICAL_BASE = {:#x}",
        KERNEL_PHYSICAL_BASE
    );

    for ph in kernel_elf.program_headers {
        if ph.p_type == elf::program_header::PT_LOAD {
            info!(
                "PT_LOAD range = {:#x?}, to address {:#x} + {:#x?}",
                ph.file_range(),
                KERNEL_PHYSICAL_BASE,
                ph.vm_range()
            );

            let dst = unsafe {
                core::slice::from_raw_parts_mut(
                    (ph.p_vaddr as usize + KERNEL_PHYSICAL_BASE) as *mut u8,
                    ph.vm_range().len(),
                )
            };

            dst.zeroize();

            unsafe {
                core::ptr::copy(
                    kernel_image.as_ptr().offset(ph.p_offset as isize),
                    dst.as_mut_ptr(),
                    ph.vm_range().len(),
                );
            }
        }
    }

    let entry_offset = kernel_elf.entry as usize;

    info!(
        "Kernel entry point: {:#x} + {:#x}",
        KERNEL_PHYSICAL_BASE, entry_offset
    );

    // Exit boot services and jump to the kernel.
    info!("Exiting UEFI boot services and jumping to the kernel");
    let mmap_size = boot_services.memory_map_size();
    let mut mmap_buf = vec![0u8; mmap_size];
    system_table
        .exit_boot_services(image_handle, &mut mmap_buf)
        .expect_success("UEFI exit boot services failed");

    // No need to relocate our kernel because it is linked as a PIE executable.
    let kernel_entry_ptr = (KERNEL_PHYSICAL_BASE + entry_offset) as *const core::ffi::c_void;
    let kernel_entry: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_entry_ptr) };

    kernel_entry();
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

    let (page_table, _) = Cr3::read();
    info!(
        "Current level 4 page table is located at: {:?}",
        page_table.start_address()
    );

    Ok(().into())
}
