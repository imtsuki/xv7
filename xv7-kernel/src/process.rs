use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

use goblin::elf;
use goblin::elf::reloc::R_X86_64_RELATIVE;

use crate::config::*;
use crate::context::Context;
use crate::fs::file::File;
use crate::paging::{AddressSpace, KernelStack, VirtAddr};
use crate::trapframe::TrapFrame;

const USER_STACK_SIZE: usize = 4096;

mod images {
    #[cfg(debug_assertions)]
    pub const INIT: &[u8] = include_bytes!("../../target/x86_64-unknown-none/debug/init");
    #[cfg(not(debug_assertions))]
    pub const INIT: &[u8] = include_bytes!("../../target/x86_64-unknown-none/release/init");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Runnable,
    Running,
    Zombie(isize),
}

static NEXT_PID: AtomicUsize = AtomicUsize::new(0);

pub struct Process {
    pub pid: usize,
    pub state: ProcessState,
    pub vm: AddressSpace,
    pub context: Context,
    pub kstack: KernelStack,
    pub fds: Vec<File>,
}

impl Process {
    pub fn new() -> Box<Self> {
        let kstack = KernelStack::alloc();
        let vm = AddressSpace::new();
        let context = Context::init_user(kstack.top());

        Box::new(Self {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            state: ProcessState::Runnable,
            vm,
            context,
            kstack,
            fds: Vec::new(),
        })
    }

    pub fn initcode() -> Box<Self> {
        let mut proc = Self::new();
        proc.exec("/init");
        proc
    }

    /// Returns a mutable reference to the user `TrapFrame` stored at the top of `self.kstack`.
    #[inline]
    pub fn trap_frame_mut(&mut self) -> &mut TrapFrame {
        let tf_addr = self.kstack.top() - size_of::<TrapFrame>() as u64;
        unsafe { &mut *tf_addr.as_mut_ptr::<TrapFrame>() }
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

        self.vm
            .map_user_range(VirtAddr::new(USER_STACK), USER_STACK_SIZE, true, false);

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

        let entry = VirtAddr::new(image_elf.entry);
        let user_sp = VirtAddr::new(USER_STACK + USER_STACK_SIZE as u64 - 16);
        self.trap_frame_mut().init_user(entry, user_sp);
    }
}
