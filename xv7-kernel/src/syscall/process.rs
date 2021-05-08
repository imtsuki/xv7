use crate::process;
use crate::{
    config::*,
    memory::{FrameAllocator, FRAME_ALLOCATOR},
};
use crate::{cpu::my_cpu, paging::VirtAddr};
use goblin::elf;
use goblin::elf::reloc::*;
use x86_64::structures::paging::{Mapper, Page};
use x86_64::structures::paging::{OffsetPageTable, PageTableFlags};
use zeroize::Zeroize;

use usyscall::error::*;

pub fn exit(code: isize) -> Result<usize> {
    let proc = process::my_proc();
    if proc.pid == 0 {
        panic!("pid 0 exited with code {}", code);
    }
    Ok(0)
}

pub fn exec(path: &str) {
    let proc = process::my_proc();

    let image = match path {
        "/init" => images::INIT,
        _ => panic!("We have no filesystem yet; executables are hardcoded"),
    };

    let image_elf = elf::Elf::parse(image).expect("Failed to parse ELF file");

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    // FIXME: we should free the previous vm and set up a new vm
    let page_table = unsafe { proc.vm.page_table() };
    let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(PAGE_OFFSET_BASE)) };

    for ph in image_elf.program_headers {
        if ph.p_type == elf::program_header::PT_LOAD {
            let page_range = {
                let start_addr = VirtAddr::new(ph.p_vaddr);
                let end_addr = start_addr + ph.p_memsz - 1u64;
                let start_page = Page::containing_address(start_addr);
                let end_page = Page::containing_address(end_addr);
                Page::range_inclusive(start_page, end_page)
            };

            let mut flags = PageTableFlags::PRESENT
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::WRITABLE;
            //if ph.is_write() {
            //}
            if !ph.is_executable() {
                flags |= PageTableFlags::NO_EXECUTE;
            }

            for page in page_range {
                let frame = frame_allocator.allocate_frame().unwrap();
                unsafe {
                    mapper
                        .map_to(page, frame, flags, &mut *frame_allocator)
                        .unwrap()
                        .flush();
                }
            }

            let dst = unsafe {
                core::slice::from_raw_parts_mut((ph.p_vaddr) as *mut u8, ph.vm_range().len())
            };

            dst.zeroize();

            dst[0..ph.file_range().len()].copy_from_slice(&image[ph.file_range()]);
        }
    }

    // allocate and map user stack.
    {
        let page = Page::containing_address(VirtAddr::new(USER_STACK));
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags = PageTableFlags::PRESENT
            | PageTableFlags::USER_ACCESSIBLE
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_EXECUTE;

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut *frame_allocator)
                .unwrap()
                .flush();
        }
    }

    // Relocate executable
    for reloc in image_elf.dynrelas.iter() {
        match reloc.r_type {
            R_X86_64_RELATIVE => {
                let addr = (reloc.r_offset) as *mut u64;
                unsafe {
                    *addr = reloc.r_addend.unwrap() as u64;
                }
            }
            _ => unimplemented!("Unhandled reloc type!"),
        }
    }

    // FIXME: magic number
    proc.set_userspace_return_address(
        VirtAddr::new(image_elf.entry),
        VirtAddr::new(USER_STACK + 4096 - 16),
    );
}

pub fn fork() -> Result<usize> {
    let _proc = process::my_proc();

    Ok(0)
}

pub fn getpid() -> Result<usize> {
    Ok(process::my_proc().pid)
}

mod images {
    pub const INIT: &'static [u8] = include_bytes!("../../../target/x86_64/debug/init");
}

pub(crate) fn r#yield() -> Result<usize> {
    let cpu = my_cpu();
    if cpu.current_process.is_some() {
        unsafe {
            cpu.switch_to_kernel();
        }
    }
    Ok(0)
}
