use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;

pub extern "x86-interrupt" fn handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT\nERROR CODE: {:#?}\n{:#?}",
        error_code, stack_frame
    );
}
