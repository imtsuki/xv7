use alloc::collections::VecDeque;

use spin::Mutex;

use crate::arch;
use crate::context::Context;
use crate::cpu;
use crate::paging;
use crate::process::{Process, ProcessState};

static RUN_QUEUE: Mutex<VecDeque<Process>> = Mutex::new(VecDeque::new());

/// Enqueues a runnable process at the tail of the global run queue.
pub fn enqueue(mut proc: Process) {
    proc.state = ProcessState::Runnable;
    arch::without_interrupts(|| {
        RUN_QUEUE.lock().push_back(proc);
    });
}

/// Main per-CPU round-robin scheduler loop.
///
/// Runs with interrupts masked (`IF == 0`) except when idling via `wait_for_interrupt`.
pub fn run() -> ! {
    arch::disable_interrupts();

    loop {
        let next = RUN_QUEUE.lock().pop_front();
        let Some(mut proc) = next else {
            arch::wait_for_interrupt();
            continue;
        };

        proc.state = ProcessState::Running;
        proc.vm.activate();
        arch::set_kernel_stack(proc.kstack.top());

        let (sched_ctx, proc_ctx) = cpu::prepare_switch_to_process(proc);
        unsafe {
            Context::switch(sched_ctx, proc_ctx);
        }

        paging::activate_kernel_page_table();

        if let Some(proc) = cpu::take_current_process() {
            match proc.state {
                ProcessState::Runnable => RUN_QUEUE.lock().push_back(proc),
                ProcessState::Zombie(code) => {
                    if proc.pid == 0 {
                        panic!("init (pid 0) exited with code {}", code);
                    }
                }
                ProcessState::Running => unreachable!("Process returned to scheduler while Running"),
            }
        }
    }
}

/// Cooperatively yields the CPU from the current process back to the scheduler.
pub fn yield_now() {
    arch::without_interrupts(|| {
        let (proc_ctx, sched_ctx) = cpu::prepare_switch_to_scheduler(ProcessState::Runnable);
        unsafe {
            Context::switch(proc_ctx, sched_ctx);
        }
    });
}

/// Terminates the current process with `code` and switches back to the scheduler.
pub fn exit_current(code: isize) -> ! {
    arch::without_interrupts(|| {
        let (proc_ctx, sched_ctx) = cpu::prepare_switch_to_scheduler(ProcessState::Zombie(code));
        unsafe {
            Context::switch(proc_ctx, sched_ctx);
        }
    });
    unreachable!("Zombie process resumed after exit")
}
