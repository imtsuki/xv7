use core::fmt;
use core::fmt::Write;
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

pub struct ConsoleGroup {
    serial: SerialConsole,
}

lazy_static! {
    pub static ref CONSOLE_GROUP: Mutex<ConsoleGroup> = Mutex::new(ConsoleGroup::new());
}

impl ConsoleGroup {
    pub fn new() -> Self {
        ConsoleGroup {
            serial: SerialConsole::new(),
        }
    }
}

impl fmt::Write for ConsoleGroup {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    CONSOLE_GROUP.lock().write_fmt(args).unwrap();
}
