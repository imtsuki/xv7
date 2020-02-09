use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

pub mod monitor;
pub mod serial;

pub use monitor::MonitorConsole;
pub use serial::SerialConsole;

/// A console device.
pub trait Console {
    fn write(&mut self, buf: &[u8]);
    /// FIXME: wrap `usize` in `Result<usize>`
    fn read(&mut self, buf: &mut [u8]) -> usize;
}

pub struct ConsoleDrivers {
    serial: SerialConsole,
}

impl ConsoleDrivers {
    pub fn new() -> Self {
        Self {
            serial: SerialConsole::new(serial::COM1),
        }
    }
}

impl fmt::Write for ConsoleDrivers {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write(s.as_bytes());
        Ok(())
    }
}

lazy_static! {
    pub static ref CONSOLE_DRIVERS: Mutex<ConsoleDrivers> = Mutex::new(ConsoleDrivers::new());
}
