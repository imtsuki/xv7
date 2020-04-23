use x86_64::structures::idt::InterruptDescriptorTable;

pub mod controller;

mod keyboard;
mod timer;
mod uart;

pub use controller::lapic::LOCAL_APIC;

pub fn install_interrupt_handlers(idt: &mut InterruptDescriptorTable) {
    idt[(T_IRQ0 + IRQ_TIMER) as usize].set_handler_fn(timer::handler);
    idt[(T_IRQ0 + IRQ_KEYBOARD) as usize].set_handler_fn(keyboard::handler);
    idt[(T_IRQ0 + IRQ_COM1) as usize].set_handler_fn(uart::handler);
}

pub fn init() {
    unsafe {
        controller::pic::disable_8259_pic();
    }

    LOCAL_APIC.lock().init();

    let mut ioapic = controller::ioapic::IoApic::default();

    ioapic.write_irq(IRQ_KEYBOARD, 0, 0);
    ioapic.write_irq(IRQ_COM1, 0, 0);
}

const T_IRQ0: u8 = 0x20;

const IRQ_TIMER: u8 = 0;
const IRQ_KEYBOARD: u8 = 1;
const IRQ_COM1: u8 = 4;
const IRQ_SPURIOUS: u8 = 31;
