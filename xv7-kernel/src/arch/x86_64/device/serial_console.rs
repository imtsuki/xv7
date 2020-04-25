use uart_16550::SerialPort;

use crate::device::console::Console;

pub const COM1: u16 = 0x3f8;

pub struct SerialConsole(SerialPort);

impl SerialConsole {
    pub fn new(port: u16) -> Self {
        Self({
            let mut serial_port = unsafe { SerialPort::new(port) };
            serial_port.init();
            serial_port
        })
    }
}

impl Console for SerialConsole {
    fn write(&mut self, buf: &[u8]) {
        for &c in buf {
            self.0.send(c);
        }
    }
}
