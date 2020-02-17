use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

pub use x86_64::instructions::interrupts::*;

mod breakpoint;
mod double_fault;
mod page_fault;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint::handler);
        idt.page_fault.set_handler_fn(page_fault::handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault::handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init() {
    IDT.load();
}
