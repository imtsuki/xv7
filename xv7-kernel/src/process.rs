use crate::context::Context;
use crate::cpu::my_cpu;
use crate::fs::file::File;
use crate::paging::{AddressSpace, VirtAddr};
use crate::{
    config::*,
    memory::{FrameAllocator, FRAME_ALLOCATOR},
};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::structures::idt::InterruptStackFrameValue;

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
        let kstack = {
            let frame = FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
            VirtAddr::new(frame.start_address().as_u64() + PAGE_OFFSET_BASE)
        };

        let stack_pointer =
            kstack + 4096usize - core::mem::size_of::<InterruptStackFrameValue>() - 8usize;

        let mut context = Context::user(stack_pointer);

        let vm = AddressSpace::new();

        context.cr3 = vm.cr3.start_address().as_u64() as usize;

        unsafe {
            stack_pointer
                .as_mut_ptr::<u64>()
                .write(interrupt_return as *const u8 as u64);
        };

        Process {
            pid: NEXT_PID.fetch_add(1, Ordering::Relaxed),
            context,
            vm,
            state: ProcessState::Spawn,
            kstack,
            fds: Vec::new(),
        }
    }

    pub fn initcode() -> Process {
        let mut p = Process::new();
        p.set_context_switch_return_address(VirtAddr::new(initcode as *const u8 as u64));
        p
    }

    pub fn intr_stack_frame(&mut self) -> &mut InterruptStackFrameValue {
        unsafe {
            &mut *((self.kstack.as_u64() + 4096
                - core::mem::size_of::<InterruptStackFrameValue>() as u64)
                as *mut InterruptStackFrameValue)
        }
    }

    fn set_context_switch_return_address(&mut self, addr: VirtAddr) {
        let stack_pointer =
            self.kstack + 4096usize - core::mem::size_of::<InterruptStackFrameValue>() - 8usize;
        unsafe {
            stack_pointer.as_mut_ptr::<u64>().write(addr.as_u64());
        };
    }

    pub fn set_userspace_return_address(
        &mut self,
        instruction_pointer: VirtAddr,
        stack_pointer: VirtAddr,
    ) {
        self.intr_stack_frame().instruction_pointer = instruction_pointer;
        self.intr_stack_frame().stack_pointer = stack_pointer;
        // FIXME: magic number
        self.intr_stack_frame().code_segment = 0x23;
        self.intr_stack_frame().stack_segment = 0x1b;
        self.intr_stack_frame().cpu_flags = 0x282;
    }
}

impl Clone for Process {
    fn clone(&self) -> Self {
        let mut p = Process::new();

        p
    }
}

#[naked]
unsafe extern "C" fn interrupt_return() {
    asm!("iretq", options(noreturn))
}

#[naked]
unsafe extern "C" fn initcode() {
    extern "C" fn initcode_exec() {
        crate::syscall::process::exec("/init");
    }
    asm!(
        "call {}",
        "iretq",
        sym initcode_exec,
        options(noreturn)
    )
}

#[naked]
unsafe extern "C" fn fork_return() {
    asm!("retq", options(noreturn));
}

pub fn my_proc() -> &'static mut Process {
    let cpu = my_cpu();
    cpu.current_process.as_mut().unwrap()
}
