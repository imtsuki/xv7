use x86_64::instructions::port::Port;

static mut MASTER: Pic = Pic::new(0x20);
static mut SLAVE: Pic = Pic::new(0xA0);

pub unsafe fn disable_8259_pic() {
    let master = &mut *(&raw mut MASTER);
    let slave = &mut *(&raw mut SLAVE);

    master.cmd.write(0x11);
    slave.cmd.write(0x11);

    master.data.write(0x20);
    slave.data.write(0x28);

    master.data.write(4);
    slave.data.write(2);

    master.data.write(1);
    slave.data.write(1);

    master.data.write(0xff);
    slave.data.write(0xff);

    master.ack();
    slave.ack();
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
