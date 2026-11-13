use crate::config::*;
use crate::io::read_file;
use crate::paging::{BootPageTable, alloc_zeroed_pages};
use boot::KernelEntry;
use goblin::elf;
use goblin::elf::reloc::*;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{VirtAddr, align_down, align_up};

/// Loads kernel image to `KERNEL_BASE` and applies PIE relocations.
pub fn load_elf(page_table: &mut BootPageTable, path: &str) -> KernelEntry {
    let (len, kernel_image) = read_file(path).expect("Could not load kernel image");
    dbg!(len);

    let kernel_elf = elf::Elf::parse(&kernel_image).expect("Failed to parse ELF file");
    dbg!(KERNEL_BASE);

    for ph in kernel_elf.program_headers {
        if ph.p_type != elf::program_header::PT_LOAD || ph.p_memsz == 0 {
            continue;
        }

        info!(
            "PT_LOAD range = {:#x?}, to address {:#x} + {:#x?}",
            ph.file_range(),
            KERNEL_BASE,
            ph.vm_range(),
        );

        let seg_offset = (ph.p_vaddr - align_down(ph.p_vaddr, Size4KiB::SIZE)) as usize;
        let page_count = (align_up((seg_offset + ph.p_memsz as usize) as u64, Size4KiB::SIZE)
            / Size4KiB::SIZE) as usize;

        info!("page_count: {}", page_count);
        let phys_base = alloc_zeroed_pages(page_count);

        let dst = unsafe {
            core::slice::from_raw_parts_mut(
                (phys_base.as_u64() as *mut u8).add(seg_offset),
                ph.file_range().len(),
            )
        };
        dst.copy_from_slice(&kernel_image[ph.file_range()]);

        let mut flags = PageTableFlags::PRESENT;
        if ph.is_write() {
            flags |= PageTableFlags::WRITABLE;
        }
        if !ph.is_executable() {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        let virt_base = VirtAddr::new(KERNEL_BASE + align_down(ph.p_vaddr, Size4KiB::SIZE));
        for i in 0..page_count as u64 {
            let page = Page::containing_address(virt_base + i * Size4KiB::SIZE);
            let frame = PhysFrame::containing_address(phys_base + i * Size4KiB::SIZE);
            page_table.map_4k(page, frame, flags);
        }
    }

    // Relocate our kernel because it is linked as a PIE executable.
    for reloc in kernel_elf.dynrelas.iter() {
        match reloc.r_type {
            R_X86_64_RELATIVE => {
                let paddr = page_table
                    .translate_addr(VirtAddr::new(KERNEL_BASE + reloc.r_offset))
                    .expect("Kernel relocation target is not mapped");
                unsafe {
                    (paddr.as_u64() as *mut u64)
                        .write_unaligned(KERNEL_BASE + reloc.r_addend.unwrap() as u64);
                }
            }
            _ => unimplemented!("Unhandled reloc type!"),
        }
    }

    assert_eq!(kernel_elf.dynrels.len(), 0);
    assert_eq!(kernel_elf.pltrelocs.len(), 0);

    KernelEntry(VirtAddr::new(KERNEL_BASE + kernel_elf.entry))
}
