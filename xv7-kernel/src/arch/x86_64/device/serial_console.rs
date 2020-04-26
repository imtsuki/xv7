use crate::arch::device::com;
use crate::device::console::Console;

pub const COM1: u16 = 0x3f8;

pub struct SerialConsole;

impl SerialConsole {
    pub fn new() -> Self {
        Self
    }
}

impl Console for SerialConsole {
    fn write(&mut self, buf: &[u8]) {
        let mut port = com::COM1.lock();
        for &c in buf {
            port.send(c);
        }
    }
}
