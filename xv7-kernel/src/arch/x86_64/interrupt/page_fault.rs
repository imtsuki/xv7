use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;

pub extern "x86-interrupt" fn handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT\ncr2: {:#?}\nerror_code: {:#?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}
