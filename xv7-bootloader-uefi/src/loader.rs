use crate::config::KERNEL_PHYSICAL_BASE;
use crate::io::read_file;
use goblin::elf;
use uefi::prelude::*;
use zeroize::Zeroize;

/// Loads kernel image to `KERNEL_PHYSICAL_BASE`.
/// Returns the entry offset with respect to `KERNEL_PHYSICAL_BASE`.
pub fn load_elf(services: &BootServices, path: &str) -> usize {
    info!("Loading kernel image from {}", path);
    let (len, kernel_image) =
        read_file(services, path).expect_success("Could not load kernel image");

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

    kernel_elf.entry as usize
}
