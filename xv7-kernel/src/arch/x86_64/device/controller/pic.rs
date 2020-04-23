use x86_64::instructions::port::Port;

static mut MASTER: Pic = Pic::new(0x20);
static mut SLAVE: Pic = Pic::new(0xA0);

pub unsafe fn disable_8259_pic() {
    MASTER.cmd.write(0x11);
    SLAVE.cmd.write(0x11);

    MASTER.data.write(0x20);
    SLAVE.data.write(0x28);

    MASTER.data.write(4);
    SLAVE.data.write(2);

    MASTER.data.write(1);
    SLAVE.data.write(1);

    MASTER.data.write(0xff);
    SLAVE.data.write(0xff);

    MASTER.ack();
    SLAVE.ack();
}

pub struct Pic {
    cmd: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    pub const fn new(port: u16) -> Self {
        Self {
            cmd: Port::new(port),
            data: Port::new(port + 1),
        }
    }

    pub fn ack(&mut self) {
        unsafe {
            self.cmd.write(0x20);
        }
    }
}
