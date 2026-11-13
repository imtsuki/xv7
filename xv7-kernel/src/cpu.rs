use core::cell::SyncUnsafeCell;

use crate::context::Context;
use crate::process::{Process, ProcessState};

pub struct Cpu {
    scheduler_context: SyncUnsafeCell<Context>,
    current_process: SyncUnsafeCell<Option<Process>>,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            scheduler_context: SyncUnsafeCell::new(Context::new()),
            current_process: SyncUnsafeCell::new(None),
        }
    }
}

/// TODO: Index by CPU ID when SMP is enabled.
static CPUS: [Cpu; 1] = [const { Cpu::new() }; 1];

#[inline]
fn current_cpu() -> &'static Cpu {
    &CPUS[0]
}

/// Executes `f` with a scoped mutable reference to the currently running `Process`.
/// Do not call context-switching functions (`scheduler::yield_now` / `exit_current`) inside `f`.
pub fn with_current_process<R>(f: impl FnOnce(&mut Process) -> R) -> R {
    let slot = unsafe { &mut *current_cpu().current_process.get() };
    let proc = slot.as_mut().expect("No process running on current CPU");
    f(proc)
}

/// Runs `proc` on the current CPU until it parks (`yield` / `exit`), then returns ownership of `proc`.
pub(crate) fn run_process(mut proc: Process) -> Process {
    let cpu = current_cpu();
    proc.state = ProcessState::Running;

    let proc_ctx = {
        let slot = unsafe { &mut *cpu.current_process.get() };
        let installed = slot.insert(proc);
        &raw const installed.context
    };

    unsafe {
        Context::switch_from(cpu.scheduler_context.get()).to(proc_ctx);
    }

    let slot = unsafe { &mut *cpu.current_process.get() };
    slot.take()
        .expect("Current process missing after returning to scheduler")
}

/// Updates the running process's state to `next_state` and switches back to the CPU scheduler.
pub(crate) fn park_current_process(next_state: ProcessState) {
    let cpu = current_cpu();

    let proc_ctx = {
        let slot = unsafe { &mut *cpu.current_process.get() };
        let proc = slot
            .as_mut()
            .expect("No running process on current CPU to park");
        proc.state = next_state;
        &raw mut proc.context
    };

    unsafe {
        Context::switch_from(proc_ctx).to(cpu.scheduler_context.get());
    }
}
