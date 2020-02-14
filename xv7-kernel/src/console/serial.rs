use super::Console;
#[cfg(target_arch = "x86_64")]
use uart_16550::SerialPort;

/// FIXME: Dummy SerialPort
#[cfg(target_arch = "aarch64")]
struct SerialPort;

#[cfg(target_arch = "aarch64")]
impl SerialPort {
    pub unsafe fn new(_: u16) -> Self {
        todo!()
    }
    pub fn init(&mut self) {
        todo!()
    }
    pub fn send(&mut self, _: u8) {
        todo!()
    }
}

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
