use super::device::SerialConsole;
use crate::ansi::{CtrlSeq, EraseParam};
use crate::device::fb_console::FramebufferConsole;

pub fn init() {
    let mut console_drivers = crate::device::console::CONSOLE_DRIVERS.lock();

    console_drivers.register_serial(SerialConsole::new());
    console_drivers.register_fb(FramebufferConsole::new());

    drop(console_drivers);

    print!(
        "{}{}{}",
        CtrlSeq::EraseDisplay(Some(EraseParam::Entire)),
        CtrlSeq::CursorPosition(None, None),
        CtrlSeq::SelectGraphicRendition(None),
    );
}
