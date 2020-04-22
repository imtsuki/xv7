use uart_16550::SerialPort;

pub static mut MASTER: Pic = Pic::new(0x20);
pub static mut SLAVE: Pic = Pic::new(0xA0);

pub unsafe fn disable_8259_pic() {
    MASTER.cmd.send(0x11);
    SLAVE.cmd.send(0x11);

    MASTER.data.send(0x20);
    SLAVE.data.send(0x28);

    MASTER.data.send(4);
    SLAVE.data.send(2);

    MASTER.data.send(1);
    SLAVE.data.send(1);

    MASTER.data.send(0xff);
    SLAVE.data.send(0xff);

    MASTER.ack();
    SLAVE.ack();
}

pub struct Pic {
    cmd: SerialPort,
    data: SerialPort,
}

impl Pic {
    pub const fn new(port: u16) -> Self {
        unsafe {
            Self {
                cmd: SerialPort::new(port),
                data: SerialPort::new(port + 1),
            }
        }
    }

    pub fn ack(&mut self) {
        self.cmd.send(0x20);
    }
}
