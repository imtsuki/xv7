//! ANSI Escape Codes.
//!
//! See
//! - [Console Virtual Terminal Sequences - Windows Console | Microsoft Docs](https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
//! - [console_codes(4) - Linux manual page](http://man7.org/linux/man-pages/man4/console_codes.4.html)
//! - [ANSI escape code - Wikipedia](https://en.wikipedia.org/wiki/ANSI_escape_code)
//! - https://www.inwap.com/pdp10/ansicode.txt

use core::fmt;

const ESC: char = '\x1B';

/// Control Sequence Introducer
///
/// Begins with [ESC](ESC) [CSI](CSI).
#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum CtrlSeq {
    /* Cursor Positioning */
    /// Cursor up by n
    ///
    /// Code: [CUU](CUU), Char: `A`
    CursorUp(Option<u16>),
    /// Cursor down by n
    ///
    /// Code: [CUD](CUD), Char: `B`
    CursorDown(Option<u16>),
    /// Cursor forward (Right) by n
    ///
    /// Code: [CUF](CUF), Char: `C`
    CursorForward(Option<u16>),
    /// Cursor backward (Left) by n
    ///
    /// Code: [CUB](CUB), Char: `D`
    CursorBackward(Option<u16>),
    /// Cursor down to beginning of nth line in the viewport
    ///
    /// Code: [CNL](CNL), Char: `E`
    CursorNextLine(Option<u16>),
    /// Cursor up to beginning of nth line in the viewport
    ///
    /// Code: [CPL](CPL), Char: `F`
    CursorPreviousLine(Option<u16>),
    /// Cursor moves to nth position horizontally in the current line
    ///
    /// Code: [CHA](CHA), Char: `G`
    CursorHorizontalAbsolute(Option<u16>),
    /// Cursor moves to x; y coordinate within the viewport,
    /// where x is the column of the y line
    ///
    /// Code: [CUP](CUP), Char: `H`
    CursorPosition(Option<u16>, Option<u16>),
    /// Cursor moves to x; y coordinate within the viewport,
    /// where x is the column of the y line
    ///
    /// Code: [HVP](HVP), Char:` f
    HorizontalVerticalPosition(Option<u16>, Option<u16>),
    /// Cursor moves to the nth position vertically in the current column
    ///
    /// Code: [VPA](VPA), Char: `d`
    VerticalPositionAbsolute(Option<u16>),
    /// Code: [VPR](VPR), Char: `e`
    VerticalPositionRelative(Option<u16>),
    /// Code: [HPA](HPA), Char: `
    HorizontalPositionAbsolute(Option<u16>),
    /// Code: [HPR](HPR), Char: `a`
    HorizontalPositionRelative(Option<u16>),
    /// Code: [SCP](SCP), Char: `s`
    SaveCursorPosition,
    /// Code: [RCP](RCP), Char: `u`
    RestoreCursorPosition,
    /* End of Cursor Positioning */
    /* Viewport Positioning */
    /// Scroll text up by n
    ///
    /// Code: [SU](SU), Char: `S`
    ScrollUp(Option<u16>),
    /// Scroll text down by n
    ///
    /// Code: [SD](SD) Char: `T`
    ScrollDown(Option<u16>),
    /* End of Viewport Positioning */
    /* Text Modification */
    /// Insert n spaces at the current cursor position, shifting
    /// all existing text to the right. Text exiting the screen to
    /// the right is removed.
    ///
    /// Code: [ICH](ICH), Char: `@`
    InsertCharacter(Option<u16>),
    /// Delete n characters at the current cursor position, shifting
    /// in space characters from the right edge of the screen.
    ///
    /// Code: [DCH](DCH), Char: `P`
    DeleteCharacter(Option<u16>),
    /// Erase n characters from the current cursor position by
    /// overwriting them with a space character.
    ///
    /// Code: [ECH](ECH), Char: `X`
    EraseCharacter(Option<u16>),
    /// Inserts n lines into the buffer at the cursor position. The line
    /// the cursor is on, and lines below it, will be shifted downwards.
    ///
    /// Code: [IL](IL), Char: `L`
    InsertLine(Option<u16>),
    /// Deletes n lines from the buffer, starting with the row the cursor is on.
    ///
    /// Code: [DL](DL), Char: `M`
    DeleteLine(Option<u16>),
    /// Replace all text in the current viewport/screen specified by `EraseParam`
    /// with space characters
    ///
    /// Code: [ED](ED), Char: `J`
    EraseDisplay(Option<EraseParam>),
    /// Replace all text on the line with the cursor specified by `EraseParam`
    /// with space characters
    ///
    /// Code: [EL](EL), Char: `K`
    EraseLine(Option<EraseParam>),
    /* End of Text Modification */
    /* Text Formatting */
    /// Set the format of the screen and text as specified by n
    ///
    /// Code: [SGR](SGR), Char: `m`
    SelectGraphicRendition(Option<u16>),
    /* End of Text Formatting */
    /* Tabs */
    /// Code: [CHT](CHT), Char: `I`
    CursorHorizontalTab(Option<u16>),
    /// Code: [CBT](CBT), Char: `Z`
    CursorBackwardsTab(Option<u16>),
    /// Code: [TBC](TBC), Char: `g`
    TabClear(Option<u16>),
}

pub const CSI: char = '[';
/* Cursor Positioning */
pub const CUU: char = 'A';
pub const CUD: char = 'B';
pub const CUF: char = 'C';
pub const CUB: char = 'D';
pub const CNL: char = 'E';
pub const CPL: char = 'F';
pub const CHA: char = 'G';
pub const CUP: char = 'H';
pub const HVP: char = 'f';
pub const VPA: char = 'd';
pub const VPR: char = 'e';
pub const HPA: char = '`';
pub const HPR: char = 'a';
pub const SCP: char = 's';
pub const RCP: char = 'u';
/* Viewport Positioning */
pub const SU: char = 'S';
pub const SD: char = 'T';
/* Text Modification */
pub const ICH: char = '@';
pub const DCH: char = 'P';
pub const ECH: char = 'X';
pub const IL: char = 'L';
pub const DL: char = 'M';
pub const ED: char = 'J';
pub const EL: char = 'K';
/* Text Formatting */
pub const SGR: char = 'm';
/* Tabs */
pub const CHT: char = 'I';
pub const CBT: char = 'Z';
pub const TBC: char = 'g';

impl fmt::Display for CtrlSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CtrlSeq::*;
        write!(f, "{}{}", ESC, CSI)?;
        match self {
            CursorUp(Some(n)) => write!(f, "{}{}", n, CUU),
            CursorUp(None) => write!(f, "{}", CUU),
            CursorDown(Some(n)) => write!(f, "{}{}", n, CUD),
            CursorDown(None) => write!(f, "{}", CUD),
            CursorForward(Some(n)) => write!(f, "{}{}", n, CUF),
            CursorForward(None) => write!(f, "{}", CUF),
            CursorBackward(Some(n)) => write!(f, "{}{}", n, CUB),
            CursorBackward(None) => write!(f, "{}", CUB),
            CursorNextLine(Some(n)) => write!(f, "{}{}", n, CNL),
            CursorNextLine(None) => write!(f, "{}", CNL),
            CursorPreviousLine(Some(n)) => write!(f, "{}{}", n, CPL),
            CursorPreviousLine(None) => write!(f, "{}", CPL),
            CursorHorizontalAbsolute(Some(n)) => write!(f, "{}{}", n, CHA),
            CursorHorizontalAbsolute(None) => write!(f, "{}", CHA),
            CursorPosition(Some(x), Some(y)) => write!(f, "{};{}{}", y, x, CUP),
            CursorPosition(Some(x), None) => write!(f, ";{}{}", x, CUP),
            CursorPosition(None, Some(y)) => write!(f, "{};{}", y, CUP),
            CursorPosition(None, None) => write!(f, "{}", CUP),
            HorizontalVerticalPosition(Some(x), Some(y)) => write!(f, "{};{}{}", y, x, HVP),
            HorizontalVerticalPosition(Some(x), None) => write!(f, ";{}{}", x, HVP),
            HorizontalVerticalPosition(None, Some(y)) => write!(f, "{};{}", y, HVP),
            HorizontalVerticalPosition(None, None) => write!(f, "{}", HVP),
            VerticalPositionAbsolute(Some(n)) => write!(f, "{}{}", n, VPA),
            VerticalPositionAbsolute(None) => write!(f, "{}", VPA),
            VerticalPositionRelative(Some(n)) => write!(f, "{}{}", n, VPR),
            VerticalPositionRelative(None) => write!(f, "{}", VPR),
            HorizontalPositionAbsolute(Some(n)) => write!(f, "{}{}", n, HPA),
            HorizontalPositionAbsolute(None) => write!(f, "{}", HPA),
            HorizontalPositionRelative(Some(n)) => write!(f, "{}{}", n, HPR),
            HorizontalPositionRelative(None) => write!(f, "{}", HPR),
            SaveCursorPosition => write!(f, "{}", SCP),
            RestoreCursorPosition => write!(f, "{}", RCP),
            ScrollUp(Some(n)) => write!(f, "{}{}", n, SU),
            ScrollUp(None) => write!(f, "{}", SU),
            ScrollDown(Some(n)) => write!(f, "{}{}", n, SD),
            ScrollDown(None) => write!(f, "{}", SD),
            InsertCharacter(Some(n)) => write!(f, "{}{}", n, ICH),
            InsertCharacter(None) => write!(f, "{}", ICH),
            DeleteCharacter(Some(n)) => write!(f, "{}{}", n, DCH),
            DeleteCharacter(None) => write!(f, "{}", DCH),
            EraseCharacter(Some(n)) => write!(f, "{}{}", n, ECH),
            EraseCharacter(None) => write!(f, "{}", ECH),
            InsertLine(Some(n)) => write!(f, "{}{}", n, IL),
            InsertLine(None) => write!(f, "{}", IL),
            DeleteLine(Some(n)) => write!(f, "{}{}", n, DL),
            DeleteLine(None) => write!(f, "{}", DL),
            EraseDisplay(Some(n)) => write!(f, "{}{}", n, ED),
            EraseDisplay(None) => write!(f, "{}", ED),
            EraseLine(Some(n)) => write!(f, "{}{}", n, EL),
            EraseLine(None) => write!(f, "{}", EL),
            SelectGraphicRendition(Some(n)) => write!(f, "{}{}", n, SGR),
            SelectGraphicRendition(None) => write!(f, "{}", SGR),
            CursorHorizontalTab(Some(n)) => write!(f, "{}{}", n, CHT),
            CursorHorizontalTab(None) => write!(f, "{}", CHT),
            CursorBackwardsTab(Some(n)) => write!(f, "{}{}", n, CBT),
            CursorBackwardsTab(None) => write!(f, "{}", CBT),
            TabClear(Some(n)) => write!(f, "{}{}", n, TBC),
            TabClear(None) => write!(f, "{}", TBC),
        }
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum EraseParam {
    FromCurrentToEnd,
    FromBeginningToCurrent,
    Entire,
}

impl fmt::Display for EraseParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EraseParam::*;
        match self {
            FromCurrentToEnd => write!(f, "0"),
            FromBeginningToCurrent => write!(f, "1"),
            Entire => write!(f, "2"),
        }
    }
}

/// C0 set of 7-bit control characters (from ANSI X3.4-1977).
#[allow(non_snake_case)]
pub mod C0 {
    /// Null filler, terminal should ignore this character
    pub const NUL: u8 = 0x00;
    /// Start of Header
    pub const SOH: u8 = 0x01;
    /// Start of Text, implied end of header
    pub const STX: u8 = 0x02;
    /// End of Text, causes some terminal to respond with ACK or NAK
    pub const ETX: u8 = 0x03;
    /// End of Transmission
    pub const EOT: u8 = 0x04;
    /// Enquiry, causes terminal to send ANSWER-BACK ID
    pub const ENQ: u8 = 0x05;
    /// Acknowledge, usually sent by terminal in response to ETX
    pub const ACK: u8 = 0x06;
    /// Bell, triggers the bell, buzzer, or beeper on the terminal
    pub const BEL: u8 = 0x07;
    /// Backspace, can be used to define overstruck characters
    pub const BS: u8 = 0x08;
    /// Horizontal Tabulation, move to next predetermined position
    pub const HT: u8 = 0x09;
    /// Linefeed, move to same position on next line (see also NL)
    pub const LF: u8 = 0x0A;
    /// Vertical Tabulation, move to next predetermined line
    pub const VT: u8 = 0x0B;
    /// Form Feed, move to next form or page
    pub const FF: u8 = 0x0C;
    /// Carriage Return, move to first character of current line
    pub const CR: u8 = 0x0D;
    /// Shift Out, switch to G1 (other half of character set)
    pub const SO: u8 = 0x0E;
    /// Shift In, switch to G0 (normal half of character set)
    pub const SI: u8 = 0x0F;
    /// Data Link Escape, interpret next control character specially
    pub const DLE: u8 = 0x10;
    /// (DC1) Terminal is allowed to resume transmitting
    pub const XON: u8 = 0x11;
    /// Device Control 2, causes ASR-33 to activate paper-tape reader
    pub const DC2: u8 = 0x12;
    /// (DC2) Terminal must pause and refrain from transmitting
    pub const XOFF: u8 = 0x13;
    /// Device Control 4, causes ASR-33 to deactivate paper-tape reader
    pub const DC4: u8 = 0x14;
    /// Negative Acknowledge, used sometimes with ETX and ACK
    pub const NAK: u8 = 0x15;
    /// Synchronous Idle, used to maintain timing in Sync communication
    pub const SYN: u8 = 0x16;
    /// End of Transmission block
    pub const ETB: u8 = 0x17;
    /// Cancel (makes VT100 abort current escape sequence if any)
    pub const CAN: u8 = 0x18;
    /// End of Medium
    pub const EM: u8 = 0x19;
    /// Substitute (VT100 uses this to display parity errors)
    pub const SUB: u8 = 0x1A;
    /// Prefix to an escape sequence
    pub const ESC: u8 = 0x1B;
    /// File Separator
    pub const FS: u8 = 0x1C;
    /// Group Separator
    pub const GS: u8 = 0x1D;
    /// Record Separator (sent by VT132 in block-transfer mode)
    pub const RS: u8 = 0x1E;
    /// Unit Separator
    pub const US: u8 = 0x1F;
    /// Delete, should be ignored by terminal
    pub const DEL: u8 = 0x7f;
}
