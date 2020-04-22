use super::super::apic::lapic::LOCAL_APIC;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(_stack_frame: &mut InterruptStackFrame) {
    LOCAL_APIC.lock().eoi();
}
