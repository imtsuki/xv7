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
}

/// TODO: Support more CPUs
static CPUS: SyncUnsafeCell<[Cpu; 1]> = SyncUnsafeCell::new([Cpu::new(); 1]);

#[inline]
fn current_cpu_ptr() -> *mut Cpu {
    unsafe { &raw mut (*CPUS.get())[0] }
}

pub fn has_current_process() -> bool {
    unsafe { (*current_cpu_ptr()).current_process.is_some() }
}

pub fn set_current_process(proc: Process) {
    unsafe {
        (*current_cpu_ptr()).current_process = Some(proc);
    }
}

/// Executes `f` with a scoped mutable reference to the currently running `Process`.
/// Do not call context-switching functions (`switch_to_process` / `yield_current_process`) inside `f`.
pub fn with_current_process<R>(f: impl FnOnce(&mut Process) -> R) -> R {
    let proc = unsafe {
        (*current_cpu_ptr())
            .current_process
            .as_mut()
            .expect("No process running on current CPU")
    };
    f(proc)
}

/// Switches from the scheduler context on this CPU to `current_process`.
/// All mutable references to `Cpu` and `Process` are dropped before `Context::switch_to`
/// to avoid aliasing `&mut` across the suspension point.
pub unsafe fn switch_to_process() {
    let (kernel_ctx_ptr, proc_ctx_ptr, kstack_top) = unsafe {
        let cpu = &mut *current_cpu_ptr();
        let proc = cpu
            .current_process
            .as_ref()
            .expect("No process to switch to");
        (
            &raw mut cpu.kernel_context,
            &raw const proc.context,
            proc.kstack_top(),
        )
    };

    unsafe {
        gdt::push_kernel_stack(kstack_top);
        Context::switch_to(kernel_ctx_ptr, proc_ctx_ptr);
    }
}

/// Yields execution from the currently running process back to the CPU scheduler context.
/// All mutable references to `Cpu` and `Process` are dropped before `Context::switch_to`.
pub fn yield_current_process() {
    let ptrs = unsafe {
        let cpu = &mut *current_cpu_ptr();
        cpu.current_process.as_mut().map(|proc| {
            (
                &raw mut proc.context,
                &raw const cpu.kernel_context,
            )
        })
    };

    if let Some((proc_ctx_ptr, kernel_ctx_ptr)) = ptrs {
        unsafe {
            gdt::pop_kernel_stack();
            Context::switch_to(proc_ctx_ptr, kernel_ctx_ptr);
        }
    }
}
