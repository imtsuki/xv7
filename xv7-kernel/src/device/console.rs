use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use spin::Mutex;

/// A console device.
pub trait Console {
    fn write(&mut self, buf: &[u8]);
}

pub struct ConsoleDrivers {
    consoles: Vec<Box<dyn Console + Send>>,
}

impl ConsoleDrivers {
    pub fn new() -> Self {
        Self {
            consoles: Vec::new(),
        }
    }

    pub fn register(&mut self, console: Box<dyn Console + Send>) {
        self.consoles.push(console)
    }

    pub fn write(&mut self, buf: &[u8]) {
        for console in &mut self.consoles {
            console.write(buf);
        }
    }
}

impl fmt::Write for ConsoleDrivers {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for console in &mut self.consoles {
            console.write(s.as_bytes());
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref CONSOLE_DRIVERS: Mutex<ConsoleDrivers> = Mutex::new(ConsoleDrivers::new());
}

lazy_static! {
    pub static ref KEYBOARD_BUFFER: ArrayQueue<u8> = ArrayQueue::new(1024);
}
