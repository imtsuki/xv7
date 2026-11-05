use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::port::PortReadOnly;

lazy_static! {
    pub static ref COM1: Mutex<SerialPort> = Mutex::new({
        let mut serial_port = unsafe { SerialPort::new(COM1_PORT) };
        serial_port.init();
        serial_port
    });
}

pub const COM1_PORT: u16 = 0x3f8;

/// Non-blocking read of a single byte from COM1 if the Line Status Register
/// indicates data is ready (bit 0 = DR).
pub fn try_receive() -> Option<u8> {
    let mut port = COM1.lock();
    let mut lsr = PortReadOnly::<u8>::new(COM1_PORT + 5);
    let status = unsafe { lsr.read() };
    if (status & 0x01) != 0 {
        Some(port.receive())
    } else {
        None
    }
}
