use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
