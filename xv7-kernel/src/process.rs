use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use goblin::elf;
use goblin::elf::reloc::R_X86_64_RELATIVE;

use crate::config::*;
use crate::context::Context;
use crate::fs::file::File;
use crate::paging::{self, AddressSpace, VirtAddr};

const KSTACK_SIZE: u64 = 4096;
const USER_STACK_SIZE: usize = 4096;

mod images {
    #[cfg(debug_assertions)]
    pub const INIT: &[u8] = include_bytes!("../../target/x86_64-unknown-none/debug/init");
    #[cfg(not(debug_assertions))]
    pub const INIT: &[u8] = include_bytes!("../../target/x86_64-unknown-none/release/init");
}

pub enum ProcessState {
    Spawn,
    Runnable,
    Running,
    Zombie,
}

static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

pub struct Process {
    pub pid: usize,
    pub state: ProcessState,
    pub vm: AddressSpace,
    pub context: Context,
    pub kstack: VirtAddr,
    pub fds: Vec<File>,
}

impl Process {
    pub fn new() -> Process {
        let kstack = paging::alloc_kernel_page();
        let kstack_top = kstack + KSTACK_SIZE;
        let vm = AddressSpace::new();
        let context = Context::init_user(kstack_top, vm.cr3_u64());

        Process {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            state: ProcessState::Spawn,
            vm,
            context,
            kstack,
            fds: Vec::new(),
        }
    }

    pub fn initcode() -> Process {
        let mut p = Process::new();
        p.exec("/init");
        p
    }

    #[inline]
    pub fn kstack_top(&self) -> VirtAddr {
        self.kstack + KSTACK_SIZE
    }

    pub fn exec(&mut self, path: &str) {
        let image = match path {
            "/init" => images::INIT,
            _ => panic!("We have no filesystem yet; executables are hardcoded"),
        };

        let image_elf = elf::Elf::parse(image).expect("Failed to parse ELF file");

        for ph in image_elf.program_headers {
            if ph.p_type == elf::program_header::PT_LOAD {
                let vaddr = VirtAddr::new(ph.p_vaddr);
                self.vm.map_user_range(
                    vaddr,
                    ph.p_memsz as usize,
                    ph.is_write(),
                    ph.is_executable(),
                );
                self.vm.copy_to_user(vaddr, &image[ph.file_range()]);
            }
        }

        // Allocate and map user stack.
        self.vm
            .map_user_range(VirtAddr::new(USER_STACK), USER_STACK_SIZE, true, false);

        // Relocate PIE executable.
        for reloc in image_elf.dynrelas.iter() {
            match reloc.r_type {
                R_X86_64_RELATIVE => {
                    let dst = VirtAddr::new(reloc.r_offset);
                    let val = reloc.r_addend.unwrap() as u64;
                    self.vm.write_user_u64(dst, val);
                }
                _ => unimplemented!("Unhandled reloc type!"),
            }
        }

        Context::set_user_return_frame(
            self.kstack_top(),
            VirtAddr::new(image_elf.entry),
            VirtAddr::new(USER_STACK + USER_STACK_SIZE as u64 - 16),
        );
    }
}

impl Clone for Process {
    fn clone(&self) -> Self {
        Process::new()
    }
}
