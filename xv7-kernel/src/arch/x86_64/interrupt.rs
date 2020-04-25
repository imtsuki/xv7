use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

pub use x86_64::instructions::interrupts::*;

pub mod controller;

mod breakpoint;
mod double_fault;
mod general_protection_fault;
mod page_fault;

mod com;
mod keyboard;
mod timer;

use controller::*;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint::handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault::handler);
        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault::handler)
                .set_stack_index(super::gdt::PAGE_FAULT_IST_INDEX);
            idt.double_fault
                .set_handler_fn(double_fault::handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[(T_IRQ0 + IRQ_TIMER) as usize].set_handler_fn(timer::handler);
        idt[(T_IRQ0 + IRQ_KEYBOARD) as usize].set_handler_fn(keyboard::handler);
        idt[(T_IRQ0 + IRQ_COM1) as usize].set_handler_fn(com::handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}
