use crate::arch::device::LOCAL_APIC;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

const COM1: u16 = 0x3f8;

pub extern "x86-interrupt" fn handler(_stack_frame: &mut InterruptStackFrame) {
    let mut data_port = Port::<u8>::new(COM1);
    let mut line_sts_port = Port::<u8>::new(COM1 + 5);
    if unsafe { line_sts_port.read() } & 0x01 != 0 {
        let byte = unsafe { data_port.read() };
        print!("{}", byte as char);
    }
    LOCAL_APIC.lock().end_of_interrupt();
}
