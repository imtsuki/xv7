use core::cell::SyncUnsafeCell;

use crate::arch::gdt;
use crate::context::Context;
use crate::process::Process;

#[repr(C)]
pub struct Cpu {
    pub kernel_context: Context,
    pub current_process: Option<Process>,
}

impl Cpu {
    pub const fn new() -> Cpu {
        Cpu {
            kernel_context: Context::new(),
            current_process: None,
        }
    }

    pub unsafe fn switch_to_process(&mut self) {
        let proc = &self.current_process.as_ref().unwrap();
        unsafe {
            let tss = &mut *gdt::TSS.get();
            tss.privilege_stack_table[1] = tss.privilege_stack_table[0];
            tss.privilege_stack_table[0] = proc.kstack + 4096u64;
            self.kernel_context.switch_to(&proc.context);
        }
    }

    pub unsafe fn switch_to_kernel(&mut self) {
        match self.current_process.as_mut() {
            Some(p) => unsafe {
                let tss = &mut *gdt::TSS.get();
                tss.privilege_stack_table[0] = tss.privilege_stack_table[1];
                p.context.switch_to(&self.kernel_context)
            },
            None => panic!("No process running"),
        }
    }
}

/// TODO: Support more cpus
static CPUS: SyncUnsafeCell<[Cpu; 1]> = SyncUnsafeCell::new([Cpu::new(); 1]);

pub fn my_cpu() -> &'static mut Cpu {
    unsafe { &mut (*CPUS.get())[0] }
}
