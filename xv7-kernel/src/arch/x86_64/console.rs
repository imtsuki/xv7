use super::device::{MonitorConsole, SerialConsole};

pub fn init() {
    let mut console_drivers = crate::device::console::CONSOLE_DRIVERS.lock();

    console_drivers.register(box MonitorConsole::new());
    console_drivers.register(box SerialConsole::new());
}
