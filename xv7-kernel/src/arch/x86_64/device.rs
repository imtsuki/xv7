use x86_64::structures::idt::InterruptDescriptorTable;

mod controller;

mod keyboard;
mod timer;

pub use controller::lapic::LOCAL_APIC;

pub fn install_interrupt_handlers(idt: &mut InterruptDescriptorTable) {
    idt[0x20].set_handler_fn(timer::handler);
    idt[0x21].set_handler_fn(keyboard::handler);
}

pub fn init() {
    unsafe {
        controller::pic::disable_8259_pic();
    }

    LOCAL_APIC.lock().init();

    let mut ioapic = controller::ioapic::IoApic::default();

    ioapic.write_irq(IRQ_KEYBOARD, 0, 0);
}

const IRQ_KEYBOARD: u8 = 1;
