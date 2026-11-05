use crate::arch::device::uart;
use crate::device::console::Console;

pub struct SerialConsole;

impl SerialConsole {
    pub const fn new() -> Self {
        Self
    }
}

impl Console for SerialConsole {
    fn write(&mut self, buf: &[u8]) {
        let mut port = uart::COM1.lock();
        for &c in buf {
            if c == b'\n' {
                port.send_raw(b'\r');
            }
            port.send_raw(c);
        }
    }
}
