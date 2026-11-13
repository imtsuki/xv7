use alloc::boxed::Box;
use core::cell::SyncUnsafeCell;
use core::mem::size_of;

use crate::arch;
use crate::context::Context;
use crate::paging;
use crate::process::{Process, ProcessState};

pub struct Cpu {
    scheduler_context: SyncUnsafeCell<Context>,
    current_process: SyncUnsafeCell<Option<Box<Process>>>,
}

const _: () = {
    assert!(size_of::<Option<Box<Process>>>() == size_of::<usize>());
};

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
    let proc = slot
        .as_deref_mut()
        .expect("No process running on current CPU");
    f(proc)
}

/// Resumes `proc` on the current CPU (activating its address space and kernel entry stack)
/// until it suspends via `suspend_current`, then restores the kernel page table and returns `proc`.
pub(crate) fn resume(mut proc: Box<Process>) -> Box<Process> {
    let cpu = current_cpu();
    proc.state = ProcessState::Running;
    proc.vm.activate();
    arch::set_kernel_stack(proc.kstack.top());

    let proc_ctx = {
        let slot = unsafe { &mut *cpu.current_process.get() };
        let installed = slot.insert(proc);
        &raw const installed.context
    };

    unsafe {
        Context::switch_from(cpu.scheduler_context.get()).to(proc_ctx);
    }

    paging::activate_kernel_page_table();

    let slot = unsafe { &mut *cpu.current_process.get() };
    slot.take()
        .expect("Current process missing after returning to scheduler")
}

/// Marks the currently running process with `next_state` and suspends it back to the scheduler.
pub(crate) fn suspend_current(next_state: ProcessState) {
    let cpu = current_cpu();

    let proc_ctx = {
        let slot = unsafe { &mut *cpu.current_process.get() };
        let proc = slot
            .as_deref_mut()
            .expect("No running process on current CPU to suspend");
        proc.state = next_state;
        &raw mut proc.context
    };

    unsafe {
        Context::switch_from(proc_ctx).to(cpu.scheduler_context.get());
    }
}
