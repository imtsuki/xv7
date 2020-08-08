use crate::config::*;
use crate::io::read_file;
use boot::KernelEntry;
use goblin::elf;
use goblin::elf::reloc::*;
use uefi::prelude::*;
use uefi::table::boot::{AllocateType, MemoryType};
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{align_down, align_up, PhysAddr, VirtAddr};
use zeroize::Zeroize;

/// Loads kernel image to `KERNEL_BASE`.
/// Returns entry's virtual address.
pub fn load_elf(
    services: &BootServices,
    page_table: &mut impl Mapper<Size4KiB>,
    allocator: &mut impl FrameAllocator<Size4KiB>,
    path: &str,
) -> KernelEntry {
    let (len, kernel_image) =
        read_file(services, path).expect_success("Could not load kernel image");

    dbg!(len);

    let kernel_elf = elf::Elf::parse(&kernel_image).expect("Failed to parse ELF file");

    dbg!(KERNEL_BASE);

    for ph in kernel_elf.program_headers {
        if ph.p_type == elf::program_header::PT_LOAD {
            info!(
                "PT_LOAD range = {:#x?}, to address {:#x} + {:#x?}",
                ph.file_range(),
                KERNEL_BASE,
                ph.vm_range(),
            );

            // Allocate pages for this segment.
            let page_count = (align_up(
                ph.p_vaddr - align_down(ph.p_vaddr, Size4KiB::SIZE) + ph.p_memsz,
                Size4KiB::SIZE,
            ) / Size4KiB::SIZE) as usize;

            info!("page_count: {}", page_count);
            let phys_addr = services
                .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, page_count)
                .expect_success("Failed to allocate pages while loading kernel segment");

            let dst = unsafe {
                core::slice::from_raw_parts_mut(
                    (ph.p_vaddr - align_down(ph.p_vaddr, Size4KiB::SIZE) + phys_addr) as *mut u8,
                    ph.vm_range().len(),
                )
            };

            dst.zeroize();

            dst[0..ph.file_range().len()].copy_from_slice(&kernel_image[ph.file_range()]);

            // Map to `KERNEL_BASE`.
            let flags = PageTableFlags::PRESENT
                | if ph.is_write() {
                    PageTableFlags::WRITABLE
                } else {
                    PageTableFlags::empty()
                }
                | if !ph.is_executable() {
                    PageTableFlags::NO_EXECUTE
                } else {
                    PageTableFlags::empty()
                };

            let start_frame = PhysFrame::containing_address(PhysAddr::new(phys_addr));
            let end_frame = PhysFrame::containing_address(
                PhysAddr::new(phys_addr) + ph.p_vaddr - align_down(ph.p_vaddr, Size4KiB::SIZE)
                    + dst.len()
                    - 1u64,
            );

            for (i, frame) in PhysFrame::range_inclusive(start_frame, end_frame).enumerate() {
                let page = Page::containing_address(
                    VirtAddr::new(ph.p_vaddr + i as u64 * Size4KiB::SIZE) + KERNEL_BASE,
                );
                unsafe {
                    page_table
                        .map_to(page, frame, flags, allocator)
                        .expect("Failed to map kernel segment")
                        .flush();
                }
            }
        }
    }

    // Relocate our kernel because it is linked as a PIE executable.
    for reloc in kernel_elf.dynrelas.iter() {
        match reloc.r_type {
            R_X86_64_RELATIVE => {
                let addr = (KERNEL_BASE + reloc.r_offset) as *mut u64;
                unsafe {
                    *addr = KERNEL_BASE + reloc.r_addend.unwrap() as u64;
                }
            }
            _ => unimplemented!("Unhandled reloc type!"),
        }
    }

    assert_eq!(kernel_elf.dynrels.len(), 0);
    assert_eq!(kernel_elf.pltrelocs.len(), 0);

    KernelEntry(VirtAddr::new(KERNEL_BASE + kernel_elf.entry))
}
