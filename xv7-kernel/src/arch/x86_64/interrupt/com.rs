use super::controller::LOCAL_APIC;
use crate::arch::device::uart;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(_stack_frame: InterruptStackFrame) {
    while let Some(byte) = uart::try_receive() {
        crate::device::tty::push_raw_byte(byte);
    }

    LOCAL_APIC.lock().end_of_interrupt();
}
