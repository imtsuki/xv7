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
    EraseInDisplay(Option<EraseParam>),
    /// Replace all text on the line with the cursor specified by `EraseParam`
    /// with space characters
    ///
    /// Code: [EL](EL), Char: `K`
    EraseInLine(Option<EraseParam>),
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

const CSI: char = '[';
/* Cursor Positioning */
const CUU: char = 'A';
const CUD: char = 'B';
const CUF: char = 'C';
const CUB: char = 'D';
const CNL: char = 'E';
const CPL: char = 'F';
const CHA: char = 'G';
const CUP: char = 'H';
const HVP: char = 'f';
const VPA: char = 'd';
const VPR: char = 'e';
const HPA: char = '`';
const HPR: char = 'a';
const SCP: char = 's';
const RCP: char = 'u';
/* Viewport Positioning */
const SU: char = 'S';
const SD: char = 'T';
/* Text Modification */
const ICH: char = '@';
const DCH: char = 'P';
const ECH: char = 'X';
const IL: char = 'L';
const DL: char = 'M';
const ED: char = 'J';
const EL: char = 'K';
/* Text Formatting */
const SGR: char = 'm';
/* Tabs */
const CHT: char = 'I';
const CBT: char = 'Z';
const TBC: char = 'g';

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
            EraseInDisplay(Some(n)) => write!(f, "{}{}", n, ED),
            EraseInDisplay(None) => write!(f, "{}", ED),
            EraseInLine(Some(n)) => write!(f, "{}{}", n, EL),
            EraseInLine(None) => write!(f, "{}", EL),
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
