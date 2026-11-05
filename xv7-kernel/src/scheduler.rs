use crate::cpu;
use crate::process::Process;

pub fn scheduler() -> ! {
    let p = Process::initcode();
    cpu::set_current_process(p);

    loop {
        println!("[scheduler] pick next process to run");
        unsafe {
            cpu::switch_to_process();
        }
    }
}
