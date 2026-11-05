use core::fmt;
use spin::Mutex;

use crate::arch::device::SerialConsole;
use crate::device::fb_console::FramebufferConsole;

/// A console output device.
pub trait Console {
    fn write(&mut self, buf: &[u8]);
}

pub struct ConsoleDrivers {
    serial: Option<SerialConsole>,
    fb: Option<FramebufferConsole>,
}

impl ConsoleDrivers {
    pub const fn new() -> Self {
        Self {
            serial: None,
            fb: None,
        }
    }

    pub fn register_serial(&mut self, serial: SerialConsole) {
        self.serial = Some(serial);
    }

    pub fn register_fb(&mut self, fb: FramebufferConsole) {
        self.fb = Some(fb);
    }

    pub fn write(&mut self, buf: &[u8]) {
        if let Some(serial) = &mut self.serial {
            serial.write(buf);
        }
        if let Some(fb) = &mut self.fb {
            fb.write(buf);
        }
    }
}

impl fmt::Write for ConsoleDrivers {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

pub static CONSOLE_DRIVERS: Mutex<ConsoleDrivers> = Mutex::new(ConsoleDrivers::new());
