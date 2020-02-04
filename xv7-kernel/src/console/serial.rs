use super::Console;
use uart_16550::SerialPort;

pub struct SerialConsole(SerialPort);

impl SerialConsole {
    pub fn new() -> Self {
        Self({
            let mut serial_port = unsafe { SerialPort::new(0x3F8) };
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
