use super::controller::LOCAL_APIC;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(_stack_frame: &mut InterruptStackFrame) {
    LOCAL_APIC.lock().end_of_interrupt();
}
