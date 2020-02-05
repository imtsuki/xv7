use super::Console;
use uart_16550::SerialPort;

pub const COM1: u16 = 0x3f8;

pub struct SerialConsole(SerialPort);

impl SerialConsole {
    pub fn new(base: u16) -> Self {
        Self({
            let mut serial_port = unsafe { SerialPort::new(base) };
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

    fn read(&mut self, _buf: &mut [u8]) -> usize {
        todo!();
    }
}
