use crate::cpu::my_cpu;
use crate::process::Process;
use crate::syscall::process::exec;

pub fn scheduler() -> ! {
    let cpu = my_cpu();
    let p = Process::new();

    cpu.current_process = Some(p);

    exec("/init");

    unsafe {
        cpu.switch_to_process();
    }

    crate::arch::idle();
}
