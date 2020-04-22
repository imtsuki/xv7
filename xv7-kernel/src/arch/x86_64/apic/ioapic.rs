use crate::config::*;
use x86_64::VirtAddr;

pub const IOAPIC_BASE: u64 = 0xFEC0_0000;

pub struct IoApic {
    sel: *mut u32,
    data: *mut u32,
}

impl IoApic {
    #[allow(unused)]
    pub unsafe fn new(addr: VirtAddr) -> Self {
        Self {
            sel: addr.as_mut_ptr(),
            data: (addr as VirtAddr + 0x10u64).as_mut_ptr(),
        }
    }

    pub unsafe fn read(&mut self, reg: u8) -> u32 {
        self.sel.write_volatile(reg as u32);
        self.data.read_volatile()
    }

    pub unsafe fn write(&mut self, reg: u8, data: u32) {
        self.sel.write_volatile(reg as u32);
        self.data.write_volatile(data);
    }

    pub fn write_irq(&mut self, irq: u8, flags: u32, apic_id: u8) {
        unsafe {
            self.write(0x10 + 2 * irq, (0x20 + irq) as u32 | flags);
            self.write(0x10 + 2 * irq + 1, (apic_id as u32) << 24);
        }
    }

    pub fn enable(&mut self, irq: u8, apic_id: u8) {
        self.write_irq(irq, 0, apic_id);
    }
}

impl Default for IoApic {
    fn default() -> Self {
        Self {
            sel: (PAGE_OFFSET_BASE + IOAPIC_BASE) as *mut u32,
            data: (PAGE_OFFSET_BASE + IOAPIC_BASE + 0x10) as *mut u32,
        }
    }
}

pub const T_IRQ0: u32 = 0x20;
