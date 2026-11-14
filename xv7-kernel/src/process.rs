use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;
use core::sync::atomic::{AtomicUsize, Ordering};

use abi::error::*;
use goblin::elf;
use goblin::elf::reloc::R_X86_64_RELATIVE;

use crate::config::*;
use crate::context::Context;
use crate::fs::file::File;
use crate::memory::{AddressSpace, MapPerm, VmAreaKind};
use crate::paging::{KernelStack, VirtAddr};
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
    pub fn new() -> Result<Box<Self>> {
        let vm = AddressSpace::new()?;
        let kstack = KernelStack::alloc();
        let context = Context::init_user(kstack.top());

        Ok(Box::new(Self {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            state: ProcessState::Runnable,
            vm,
            context,
            kstack,
            fds: Vec::new(),
        }))
    }

    pub fn initcode() -> Box<Self> {
        let mut proc = Self::new().expect("Failed to allocate init process");
        proc.exec("/init").expect("Failed to load /init");
        proc
    }

    /// Returns a mutable reference to the user `TrapFrame` stored at the top of `self.kstack`.
    #[inline]
    pub fn trap_frame_mut(&mut self) -> &mut TrapFrame {
        let tf_addr = self.kstack.top() - size_of::<TrapFrame>() as u64;
        unsafe { &mut *tf_addr.as_mut_ptr::<TrapFrame>() }
    }

    pub fn exec(&mut self, path: &str) -> Result<()> {
        let image = match path {
            "/init" => images::INIT,
            _ => return Err(Error::new(ENOENT)),
        };

        let image_elf = elf::Elf::parse(image).map_err(|_| Error::new(ENOEXEC))?;

        let mut new_vm = AddressSpace::new()?;

        for ph in image_elf.program_headers {
            if ph.p_type != elf::program_header::PT_LOAD || ph.p_memsz == 0 {
                continue;
            }
            let mut perm = MapPerm::U | MapPerm::R;
            if ph.is_write() {
                perm |= MapPerm::W;
            }
            if ph.is_executable() {
                perm |= MapPerm::X;
            }

            new_vm.map_region(ph.p_vaddr, ph.p_memsz as usize, perm, VmAreaKind::Elf)?;
            new_vm.init_bytes(ph.p_vaddr, &image[ph.file_range()])?;
        }

        new_vm.map_region(
            USER_STACK,
            USER_STACK_SIZE,
            MapPerm::U | MapPerm::R | MapPerm::W,
            VmAreaKind::Stack,
        )?;

        for reloc in image_elf.dynrelas.iter() {
            match reloc.r_type {
                R_X86_64_RELATIVE => {
                    let addend = reloc.r_addend.ok_or_else(|| Error::new(ENOEXEC))? as u64;
                    new_vm.init_bytes(reloc.r_offset, &addend.to_ne_bytes())?;
                }
                _ => return Err(Error::new(ENOEXEC)),
            }
        }

        if self.vm.is_active() {
            new_vm.activate();
        }
        self.vm = new_vm;

        let entry = VirtAddr::new(image_elf.entry);
        let user_sp = VirtAddr::new(USER_STACK + USER_STACK_SIZE as u64 - 16);
        self.trap_frame_mut().init_user(entry, user_sp);
        Ok(())
    }
}
