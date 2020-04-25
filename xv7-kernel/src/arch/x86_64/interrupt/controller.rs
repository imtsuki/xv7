pub mod ioapic;
pub mod lapic;
pub mod pic;

pub use lapic::LOCAL_APIC;

pub fn init() {
    unsafe {
        pic::disable_8259_pic();
    }

    lapic::LOCAL_APIC.lock().init();

    let mut ioapic = ioapic::IoApic::default();

    ioapic.write_irq(IRQ_KEYBOARD, 0, 0);
    ioapic.write_irq(IRQ_COM1, 0, 0);
}

pub const T_IRQ0: u8 = 0x20;

pub const IRQ_TIMER: u8 = 0;
pub const IRQ_KEYBOARD: u8 = 1;
pub const IRQ_COM1: u8 = 4;
pub const IRQ_SPURIOUS: u8 = 31;
