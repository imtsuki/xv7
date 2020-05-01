use super::device::{MonitorConsole, SerialConsole};

use crate::ansi::{CtrlSeq, EraseParam};

pub fn init() {
    let mut console_drivers = crate::device::console::CONSOLE_DRIVERS.lock();

    console_drivers.register(box MonitorConsole::new());
    console_drivers.register(box SerialConsole::new());

    drop(console_drivers);

    print!(
        "{}{}{}",
        CtrlSeq::EraseDisplay(Some(EraseParam::Entire)),
        CtrlSeq::CursorPosition(None, None),
        CtrlSeq::SelectGraphicRendition(None),
    );
}
