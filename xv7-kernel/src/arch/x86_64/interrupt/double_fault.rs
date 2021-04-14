use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\nerror_code: {:#x}\n{:#?}",
        error_code, stack_frame
    );
}
