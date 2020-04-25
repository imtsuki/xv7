use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::VirtAddr;

use super::*;
use crate::config::*;

pub const LOCAL_APIC_BASE: u64 = 0xFEE0_0000;

pub struct LocalApic {
    base: VirtAddr,
}

impl LocalApic {
    pub unsafe fn new(addr: VirtAddr) -> Self {
        Self { base: addr }
    }

    pub unsafe fn read(&self, reg: u32) -> u32 {
        (self.base + reg as u64).as_ptr::<u32>().read_volatile()
    }

    pub unsafe fn write(&mut self, reg: u32, value: u32) {
        (self.base + reg as u64)
            .as_mut_ptr::<u32>()
            .write_volatile(value);
        // Wait for completion
        self.read(0x20);
    }

    pub fn init(&mut self) {
        unsafe {
            self.write(LAPIC_SVR, 0x100 + (T_IRQ0 + IRQ_SPURIOUS) as u32);

            self.write(LAPIC_TDCR, X1); // Timer divided by 1
            self.write(LAPIC_TIMER, PERIODIC | (T_IRQ0 + IRQ_TIMER) as u32);
            self.write(LAPIC_TICR, 10000000);

            self.write(LAPIC_EOI, 0);

            self.write(LAPIC_TPR, 0);
        }
    }

    pub fn end_of_interrupt(&mut self) {
        unsafe {
            self.write(LAPIC_EOI, 0);
        }
    }
}

lazy_static! {
    pub static ref LOCAL_APIC: Mutex<LocalApic> =
        Mutex::new(unsafe { LocalApic::new(VirtAddr::new(PAGE_OFFSET_BASE + LOCAL_APIC_BASE)) });
}

/// Local APIC ID
pub const LAPIC_ID: u32 = 0x0020;
/// Local APIC Version
pub const LAPIC_VER: u32 = 0x0030;
/// Task Priority
pub const LAPIC_TPR: u32 = 0x0080;
/// Arbitration Priority
pub const LAPIC_APR: u32 = 0x0090;
/// Processor Priority
pub const LAPIC_PPR: u32 = 0x00a0;
/// EOI
pub const LAPIC_EOI: u32 = 0x00b0;
/// Remote Read
pub const LAPIC_RRD: u32 = 0x00c0;
/// Logical Destination
pub const LAPIC_LDR: u32 = 0x00d0;
/// Destination Format
pub const LAPIC_DFR: u32 = 0x00e0;
/// Spurious Interrupt Vector
pub const LAPIC_SVR: u32 = 0x00f0;
/// In-Service (8 registers)
pub const LAPIC_ISR: u32 = 0x0100;
/// Trigger Mode (8 registers)
pub const LAPIC_TMR: u32 = 0x0180;
/// Interrupt Request (8 registers)
pub const LAPIC_IRR: u32 = 0x0200;
/// Error Status
pub const LAPIC_ESR: u32 = 0x0280;
/// Interrupt Command
pub const LAPIC_ICRLO: u32 = 0x0300;
/// Interrupt Command [63:32]
pub const LAPIC_ICRHI: u32 = 0x0310;
/// LVT Timer
pub const LAPIC_TIMER: u32 = 0x0320;
/// LVT Thermal Sensor
pub const LAPIC_THERMAL: u32 = 0x0330;
/// LVT Performance Counter
pub const LAPIC_PERF: u32 = 0x0340;
/// LVT LINT0
pub const LAPIC_LINT0: u32 = 0x0350;
/// LVT LINT1
pub const LAPIC_LINT1: u32 = 0x0360;
/// LVT Error
pub const LAPIC_ERROR: u32 = 0x0370;
/// Initial Count (for Timer)
pub const LAPIC_TICR: u32 = 0x0380;
/// Current Count (for Timer)
pub const LAPIC_TCCR: u32 = 0x0390;
/// Divide Configuration (for Timer)
pub const LAPIC_TDCR: u32 = 0x03e0;

pub const X1: u32 = 0x0000000B;
pub const PERIODIC: u32 = 0x00020000;
