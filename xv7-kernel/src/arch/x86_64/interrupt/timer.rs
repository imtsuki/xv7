use crate::cpu::my_cpu;

use super::controller::LOCAL_APIC;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(_stack_frame: InterruptStackFrame) {
    unsafe { asm!("cli") };
    LOCAL_APIC.lock().end_of_interrupt();

    let cpu = my_cpu();
    if cpu.current_process.is_some() {
        //unsafe {
        // cpu.switch_to_kernel();
        //}
    }
    unsafe { asm!("sti") };
}
