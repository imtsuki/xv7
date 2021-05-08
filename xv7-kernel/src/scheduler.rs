use crate::cpu::my_cpu;
use crate::process::Process;

pub fn scheduler() -> ! {
    let cpu = my_cpu();
    let p = Process::initcode();

    cpu.current_process = Some(p);

    loop {
        println!("[scheduler] pick next process to run");
        unsafe {
            cpu.switch_to_process();
        }
    }
}
