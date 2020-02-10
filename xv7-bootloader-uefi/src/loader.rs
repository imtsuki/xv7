use crate::config::*;
use crate::io::read_file;
use boot::{PhysAddr, PhysMemoryDescriptor, PhysMemoryType};
use core::convert::Into;
use goblin::elf;
use goblin::elf::reloc::*;
use uefi::prelude::*;
use x86_64::align_up;
use zeroize::Zeroize;

#[derive(Debug, Copy, Clone)]
pub struct KernelEntry(usize);

impl Into<usize> for KernelEntry {
    fn into(self) -> usize {
        self.0
    }
}

/// Loads kernel image to `KERNEL_PHYSICAL_BASE`.
/// Returns the entry offset with respect to `KERNEL_PHYSICAL_BASE`.
pub fn load_elf(services: &BootServices, path: &str) -> (KernelEntry, PhysMemoryDescriptor) {
    let (len, kernel_image) =
        read_file(services, path).expect_success("Could not load kernel image");

    dbg!(len);

    let kernel_elf = elf::Elf::parse(&kernel_image).expect("Failed to parse ELF file");

    dbg!(KERNEL_PHYSICAL_BASE, KERNEL_VIRTUAL_BASE);

    let mut kernel_upper_bound = 0;

    for ph in kernel_elf.program_headers {
        if ph.p_type == elf::program_header::PT_LOAD {
            info!(
                "PT_LOAD range = {:#x?}, to address {:#x} + {:#x?}",
                ph.file_range(),
                KERNEL_PHYSICAL_BASE,
                ph.vm_range()
            );

            if ph.vm_range().end > kernel_upper_bound {
                kernel_upper_bound = ph.vm_range().end;
            }

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
                    ph.file_range().len(),
                );
            }
        }
    }

    // Relocate our kernel because it is linked as a PIE executable.
    for reloc in kernel_elf.dynrelas.iter() {
        match reloc.r_type {
            R_X86_64_RELATIVE => {
                let addr = (KERNEL_PHYSICAL_BASE + reloc.r_offset as usize) as *mut u64;
                unsafe {
                    *addr = KERNEL_VIRTUAL_BASE as u64 + reloc.r_addend.unwrap() as u64;
                }
            }
            _ => unimplemented!("Unhandled reloc type!"),
        }
    }

    assert_eq!(kernel_elf.dynrels.len(), 0);
    assert_eq!(kernel_elf.pltrelocs.len(), 0);

    (
        KernelEntry(KERNEL_VIRTUAL_BASE + kernel_elf.entry as usize),
        PhysMemoryDescriptor {
            memory_type: PhysMemoryType::Kernel,
            base: PhysAddr::new(KERNEL_PHYSICAL_BASE as u64),
            page_count: align_up(kernel_upper_bound as u64, 4096) as usize / 4096,
        },
    )
}
