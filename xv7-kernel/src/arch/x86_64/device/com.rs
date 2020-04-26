use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref COM1: Mutex<SerialPort> = Mutex::new({
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        serial_port
    });
}
