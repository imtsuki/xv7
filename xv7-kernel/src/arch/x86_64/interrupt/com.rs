use super::controller::LOCAL_APIC;
use crate::arch::device::com;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(_stack_frame: &mut InterruptStackFrame) {
    let byte = {
        let mut port = com::COM1.lock();
        port.receive()
    };

    // http://web.mit.edu/broder/Public/fixing-jos-serial.txt
    let byte = match byte {
        b'\r' => b'\n',
        b'\x7f' => b'\x08',
        _ => byte,
    };

    print!("{}", byte as char);

    LOCAL_APIC.lock().end_of_interrupt();
}
