use core::cell::SyncUnsafeCell;

use crate::context::Context;
use crate::process::{Process, ProcessState};

pub struct Cpu {
    scheduler_context: Context,
    current_process: Option<Process>,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            scheduler_context: Context::new(),
            current_process: None,
        }
    }
}

/// TODO: Index by CPU ID when SMP is enabled.
static CPUS: SyncUnsafeCell<[Cpu; 1]> = SyncUnsafeCell::new([Cpu::new(); 1]);

#[inline]
fn current_cpu_ptr() -> *mut Cpu {
    unsafe { &raw mut (*CPUS.get())[0] }
}

/// Executes `f` with a scoped mutable reference to the currently running `Process`.
/// Do not call context-switching functions (`scheduler::yield_now` / `exit_current`) inside `f`.
pub fn with_current_process<R>(f: impl FnOnce(&mut Process) -> R) -> R {
    let proc = unsafe {
        (*current_cpu_ptr())
            .current_process
            .as_mut()
            .expect("No process running on current CPU")
    };
    f(proc)
}

/// Installs `proc` on the current CPU and returns raw `(scheduler_ctx, proc_ctx)` pointers
/// after dropping all temporary `&mut` references (avoiding aliasing `&mut` across `switch_to`).
pub(crate) fn prepare_switch_to_process(proc: Process) -> (*mut Context, *const Context) {
    unsafe {
        let cpu = &mut *current_cpu_ptr();
        let proc_ref = cpu.current_process.insert(proc);
        (&raw mut cpu.scheduler_context, &raw const proc_ref.context)
    }
}

/// Updates the current process state to `next_state` and returns raw `(proc_ctx, scheduler_ctx)`
/// pointers after dropping all temporary `&mut` references.
pub(crate) fn prepare_switch_to_scheduler(
    next_state: ProcessState,
) -> (*mut Context, *const Context) {
    unsafe {
        let cpu = &mut *current_cpu_ptr();
        let proc = cpu
            .current_process
            .as_mut()
            .expect("No running process to switch from");
        proc.state = next_state;
        (&raw mut proc.context, &raw const cpu.scheduler_context)
    }
}

/// Takes ownership of the process that just yielded or exited on this CPU.
pub(crate) fn take_current_process() -> Option<Process> {
    unsafe { (*current_cpu_ptr()).current_process.take() }
}
