use alloc::boxed::Box;

use super::device::{MonitorConsole, SerialConsole};

use crate::ansi::{CtrlSeq, EraseParam};

pub fn init() {
    let mut console_drivers = crate::device::console::CONSOLE_DRIVERS.lock();

    console_drivers.register(Box::new(MonitorConsole::new()));
    console_drivers.register(Box::new(SerialConsole::new()));

    drop(console_drivers);

    print!(
        "{}{}{}",
        CtrlSeq::EraseDisplay(Some(EraseParam::Entire)),
        CtrlSeq::CursorPosition(None, None),
        CtrlSeq::SelectGraphicRendition(None),
    );
}
