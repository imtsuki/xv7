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
}

pub struct ConsoleDrivers {
    serial: SerialConsole,
    monitor: MonitorConsole,
}

impl ConsoleDrivers {
    pub fn new() -> Self {
        Self {
            serial: SerialConsole::new(serial::COM1),
            monitor: MonitorConsole::new(),
        }
    }
}

impl fmt::Write for ConsoleDrivers {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write(s.as_bytes());
        self.monitor.write(s.as_bytes());
        Ok(())
    }
}

lazy_static! {
    pub static ref CONSOLE_DRIVERS: Mutex<ConsoleDrivers> = Mutex::new(ConsoleDrivers::new());
}
