use core::cell::SyncUnsafeCell;

use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, iso_8859_1::FONT_8X13},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Baseline, Text},
};
use vte::{Params, Parser, Perform};

use crate::ansi::{self, C0};
use crate::device::console::Console;
use crate::video::{GOP_DISPLAY, GopDisplay};

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;
const FONT_HEIGHT: usize = 13;

const MAX_COLS: usize = 240;
const MAX_ROWS: usize = 100;

const DEFAULT_FG_PX: u32 = 0x00FF_FFFF;
const DEFAULT_BG_PX: u32 = 0x0000_0000;

/// Packs `(ch, fg_px, bg_px)` into a 64-bit integer so cell comparisons in
/// `scroll_up_one_line` compile to a single register compare even in `opt-level = 0`,
/// and `0u64` represents a default blank cell (`' '` on black background) in `.bss`.
#[inline(always)]
const fn pack_cell(ch: char, fg_px: u32, bg_px: u32) -> u64 {
    if ch == ' ' || ch == '\0' {
        if bg_px == 0 {
            return 0;
        }
        return ((bg_px as u64 & 0xFF_FFFF) << 40) | (b' ' as u64);
    }
    let ch_u16 = if (ch as u32) <= 0xFFFF {
        ch as u64
    } else {
        b'?' as u64
    };
    ch_u16 | ((fg_px as u64 & 0xFF_FFFF) << 16) | ((bg_px as u64 & 0xFF_FFFF) << 40)
}

#[inline(always)]
const fn unpack_cell(cell: u64) -> (u16, u32, u32) {
    if cell == 0 {
        return (b' ' as u16, DEFAULT_FG_PX, DEFAULT_BG_PX);
    }
    let ch = (cell & 0xFFFF) as u16;
    let fg_px = ((cell >> 16) & 0xFF_FFFF) as u32;
    let bg_px = ((cell >> 40) & 0xFF_FFFF) as u32;
    (ch, fg_px, bg_px)
}

static SHADOW_GRID: SyncUnsafeCell<[[u64; MAX_COLS]; MAX_ROWS]> =
    SyncUnsafeCell::new([[0u64; MAX_COLS]; MAX_ROWS]);

struct GlyphCache {
    bitmaps: [[u8; FONT_HEIGHT]; 128],
    valid: [bool; 128],
}

static GLYPH_CACHE: SyncUnsafeCell<GlyphCache> = SyncUnsafeCell::new(GlyphCache {
    bitmaps: [[0; FONT_HEIGHT]; 128],
    valid: [false; 128],
});

struct GlyphExtractor([u8; FONT_HEIGHT]);

impl OriginDimensions for GlyphExtractor {
    fn size(&self) -> Size {
        Size::new(CHAR_WIDTH as u32, FONT_HEIGHT as u32)
    }
}

impl DrawTarget for GlyphExtractor {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0
                && (coord.x as usize) < CHAR_WIDTH
                && coord.y >= 0
                && (coord.y as usize) < FONT_HEIGHT
                && color != Rgb888::BLACK
            {
                self.0[coord.y as usize] |= 0x80 >> (coord.x as usize);
            }
        }
        Ok(())
    }
}

fn get_ascii_glyph(ch_u16: u16) -> Option<&'static [u8; FONT_HEIGHT]> {
    let idx = ch_u16 as usize;
    if idx >= 128 {
        return None;
    }
    let cache = unsafe { &mut *GLYPH_CACHE.get() };
    if !cache.valid[idx] {
        let mut extractor = GlyphExtractor([0; FONT_HEIGHT]);
        let buf = [idx as u8];
        let s = unsafe { core::str::from_utf8_unchecked(&buf) };
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_8X13)
            .text_color(Rgb888::WHITE)
            .background_color(Rgb888::BLACK)
            .build();
        Text::with_baseline(s, Point::zero(), style, Baseline::Top)
            .draw(&mut extractor)
            .ok();
        cache.bitmaps[idx] = extractor.0;
        cache.valid[idx] = true;
    }
    Some(&cache.bitmaps[idx])
}

struct ConsoleState {
    pos: (usize, usize),
    size: (usize, usize),
    row_len: [usize; MAX_ROWS],
    fg_px: u32,
    bg_px: u32,
}

impl ConsoleState {
    const fn new() -> Self {
        Self {
            pos: (0, 0),
            size: (0, 0),
            row_len: [0; MAX_ROWS],
            fg_px: DEFAULT_FG_PX,
            bg_px: DEFAULT_BG_PX,
        }
    }

    fn ensure_size(&mut self, display: &GopDisplay) {
        if self.size == (0, 0) {
            let (width, height) = display.resolution();
            let rows = (height / CHAR_HEIGHT).min(MAX_ROWS);
            let cols = (width / CHAR_WIDTH).min(MAX_COLS);
            self.size = (rows, cols);
        }
    }
}

pub struct FramebufferConsole {
    parser: Parser,
    state: ConsoleState,
}

impl FramebufferConsole {
    pub fn new() -> Self {
        let mut state = ConsoleState::new();
        if let Some(display) = GOP_DISPLAY.lock().as_ref() {
            state.ensure_size(display);
        }
        Self {
            parser: Parser::new(),
            state,
        }
    }
}

impl Console for FramebufferConsole {
    fn write(&mut self, buf: &[u8]) {
        let mut display_guard = GOP_DISPLAY.lock();
        let Some(display) = display_guard.as_mut() else {
            return;
        };
        self.state.ensure_size(display);
        if self.state.size.0 == 0 || self.state.size.1 == 0 {
            return;
        }

        let grid = unsafe { &mut *SHADOW_GRID.get() };
        let mut performer = Performer {
            state: &mut self.state,
            display,
            grid,
        };
        self.parser.advance(&mut performer, buf);
    }
}

struct Performer<'a> {
    state: &'a mut ConsoleState,
    display: &'a mut GopDisplay,
    grid: &'a mut [[u64; MAX_COLS]; MAX_ROWS],
}

impl<'a> Performer<'a> {
    fn draw_cell(&mut self, row: usize, col: usize, cell: u64) {
        let x = col * CHAR_WIDTH;
        let y = row * CHAR_HEIGHT;

        if cell == 0 {
            self.display
                .draw_solid_rect_packed(x, y, CHAR_WIDTH, CHAR_HEIGHT, 0);
            return;
        }

        let (ch_u16, fg_px, bg_px) = unpack_cell(cell);
        if ch_u16 == b' ' as u16 || ch_u16 == 0 {
            self.display
                .draw_solid_rect_packed(x, y, CHAR_WIDTH, CHAR_HEIGHT, bg_px);
            return;
        }

        if let Some(bitmap) = get_ascii_glyph(ch_u16) {
            self.display.draw_glyph_8x16(x, y, bitmap, fg_px, bg_px);
            return;
        }

        if let Some(ch) = char::from_u32(ch_u16 as u32) {
            let fg = Rgb888::new(
                ((fg_px >> 16) & 0xFF) as u8,
                ((fg_px >> 8) & 0xFF) as u8,
                (fg_px & 0xFF) as u8,
            );
            let bg = Rgb888::new(
                ((bg_px >> 16) & 0xFF) as u8,
                ((bg_px >> 8) & 0xFF) as u8,
                (bg_px & 0xFF) as u8,
            );
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            let style = MonoTextStyleBuilder::new()
                .font(&FONT_8X13)
                .text_color(fg)
                .background_color(bg)
                .build();
            Text::with_baseline(s, Point::new(x as i32, y as i32), style, Baseline::Top)
                .draw(self.display)
                .ok();
            if CHAR_HEIGHT > FONT_HEIGHT {
                self.display.draw_solid_rect_packed(
                    x,
                    y + FONT_HEIGHT,
                    CHAR_WIDTH,
                    CHAR_HEIGHT - FONT_HEIGHT,
                    bg_px,
                );
            }
        }
    }

    fn set_cell(&mut self, row: usize, col: usize, cell: u64) {
        if row < self.state.size.0 && col < self.state.size.1 {
            if cell != 0 && col + 1 > self.state.row_len[row] {
                self.state.row_len[row] = col + 1;
            }
            if self.grid[row][col] != cell {
                self.grid[row][col] = cell;
                self.draw_cell(row, col, cell);
            }
        }
    }

    fn scroll_up_one_line(&mut self) {
        let (rows, cols) = self.state.size;
        if rows == 0 || cols == 0 {
            return;
        }

        let mut max_dirty_col = 0;
        let mut r = 0;
        while r < rows {
            if self.state.row_len[r] > max_dirty_col {
                max_dirty_col = self.state.row_len[r];
            }
            r += 1;
        }

        if rows > 1 {
            self.grid.copy_within(1..rows, 0);
            self.state.row_len.copy_within(1..rows, 0);
        }

        let last = rows - 1;
        let last_len = self.state.row_len[last].min(cols);
        let blank = pack_cell(' ', self.state.fg_px, self.state.bg_px);
        let mut c = 0;
        while c < last_len {
            self.grid[last][c] = blank;
            c += 1;
        }
        self.state.row_len[last] = if blank == 0 { 0 } else { cols };

        let clear_px_width = if blank == 0 {
            max_dirty_col.min(cols) * CHAR_WIDTH
        } else {
            cols * CHAR_WIDTH
        };
        self.display
            .scroll_up_packed(CHAR_HEIGHT, clear_px_width, self.state.bg_px);
    }

    fn linefeed(&mut self) {
        let (rows, _) = self.state.size;
        let (mut row, _) = self.state.pos;
        row += 1;
        if row >= rows {
            self.scroll_up_one_line();
            row = rows - 1;
        }
        self.state.pos = (row, 0);
    }

    fn carriage_return(&mut self) {
        self.state.pos.1 = 0;
    }

    fn backspace(&mut self) {
        self.state.pos.1 = self.state.pos.1.saturating_sub(1);
    }

    fn advance_cursor(&mut self) {
        let (rows, cols) = self.state.size;
        let (mut row, mut col) = self.state.pos;
        col += 1;
        if col >= cols {
            col = 0;
            row += 1;
            if row >= rows {
                self.scroll_up_one_line();
                row = rows - 1;
            }
        }
        self.state.pos = (row, col);
    }

    fn clear_entire_display(&mut self) {
        let (rows, cols) = self.state.size;
        let blank = pack_cell(' ', self.state.fg_px, self.state.bg_px);
        let mut r = 0;
        while r < rows {
            let max_c = if blank == 0 {
                self.state.row_len[r].min(cols)
            } else {
                cols
            };
            let mut c = 0;
            while c < max_c {
                self.grid[r][c] = blank;
                c += 1;
            }
            self.state.row_len[r] = if blank == 0 { 0 } else { cols };
            r += 1;
        }
        let (width, height) = self.display.resolution();
        self.display
            .draw_solid_rect_packed(0, 0, width, height, self.state.bg_px);
    }

    fn clear_range(&mut self, start_row: usize, start_col: usize, end_row: usize, end_col: usize) {
        let (rows, cols) = self.state.size;
        let blank = pack_cell(' ', self.state.fg_px, self.state.bg_px);
        for r in start_row..=end_row.min(rows.saturating_sub(1)) {
            let c_start = if r == start_row { start_col } else { 0 };
            let c_end = if r == end_row {
                end_col.min(cols.saturating_sub(1))
            } else {
                cols.saturating_sub(1)
            };
            for c in c_start..=c_end {
                self.set_cell(r, c, blank);
            }
        }
    }

    const fn ansi_color_px(code: u16, bright: bool) -> u32 {
        match (code, bright) {
            (0, false) => 0x0000_0000,
            (1, false) => 0x00AA_0000,
            (2, false) => 0x0000_AA00,
            (3, false) => 0x00AA_5500,
            (4, false) => 0x0000_00AA,
            (5, false) => 0x00AA_00AA,
            (6, false) => 0x0000_AAAA,
            (7, false) => 0x00AA_AAAA,
            (0, true) => 0x0055_5555,
            (1, true) => 0x00FF_5555,
            (2, true) => 0x0055_FF55,
            (3, true) => 0x00FF_FF55,
            (4, true) => 0x0055_55FF,
            (5, true) => 0x00FF_55FF,
            (6, true) => 0x0055_FFFF,
            (_, _) => 0x00FF_FFFF,
        }
    }

    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            self.state.fg_px = DEFAULT_FG_PX;
            self.state.bg_px = DEFAULT_BG_PX;
            return;
        }
        for subparams in params.iter() {
            let code = subparams.first().copied().unwrap_or(0);
            match code {
                0 => {
                    self.state.fg_px = DEFAULT_FG_PX;
                    self.state.bg_px = DEFAULT_BG_PX;
                }
                30..=37 => self.state.fg_px = Self::ansi_color_px(code - 30, false),
                39 => self.state.fg_px = DEFAULT_FG_PX,
                40..=47 => self.state.bg_px = Self::ansi_color_px(code - 40, false),
                49 => self.state.bg_px = DEFAULT_BG_PX,
                90..=97 => self.state.fg_px = Self::ansi_color_px(code - 90, true),
                100..=107 => self.state.bg_px = Self::ansi_color_px(code - 100, true),
                _ => {}
            }
        }
    }
}

impl<'a> Perform for Performer<'a> {
    fn print(&mut self, c: char) {
        let (row, col) = self.state.pos;
        let cell = pack_cell(c, self.state.fg_px, self.state.bg_px);
        self.set_cell(row, col, cell);
        self.advance_cursor();
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            C0::CR => self.carriage_return(),
            C0::LF => self.linefeed(),
            C0::BS => self.backspace(),
            C0::HT => {
                let (_, col) = self.state.pos;
                let next_tab = ((col / 8) + 1) * 8;
                let spaces = (next_tab - col).min(self.state.size.1.saturating_sub(col));
                for _ in 0..spaces {
                    self.print(' ');
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        params: &Params,
        intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if !intermediates.is_empty() {
            return;
        }
        let (rows, cols) = self.state.size;
        let (row, col) = self.state.pos;

        let mut iter = params.iter();
        let p0 = iter.next().and_then(|s| s.first().copied()).unwrap_or(0);
        let p1 = iter.next().and_then(|s| s.first().copied()).unwrap_or(0);

        match action {
            ansi::ED => match p0 {
                0 => {
                    if row == 0 && col == 0 {
                        self.clear_entire_display();
                    } else {
                        self.clear_range(row, col, rows.saturating_sub(1), cols.saturating_sub(1));
                    }
                }
                1 => self.clear_range(0, 0, row, col),
                2 | 3 => self.clear_entire_display(),
                _ => {}
            },
            ansi::EL => match p0 {
                0 => self.clear_range(row, col, row, cols.saturating_sub(1)),
                1 => self.clear_range(row, 0, row, col),
                2 => self.clear_range(row, 0, row, cols.saturating_sub(1)),
                _ => {}
            },
            ansi::CUP | ansi::HVP => {
                let target_row = (p0.max(1) as usize - 1).min(rows.saturating_sub(1));
                let target_col = (p1.max(1) as usize - 1).min(cols.saturating_sub(1));
                self.state.pos = (target_row, target_col);
            }
            ansi::CUU => {
                let n = p0.max(1) as usize;
                self.state.pos.0 = row.saturating_sub(n);
            }
            ansi::CUD => {
                let n = p0.max(1) as usize;
                self.state.pos.0 = (row + n).min(rows.saturating_sub(1));
            }
            ansi::CUF => {
                let n = p0.max(1) as usize;
                self.state.pos.1 = (col + n).min(cols.saturating_sub(1));
            }
            ansi::CUB => {
                let n = p0.max(1) as usize;
                self.state.pos.1 = col.saturating_sub(n);
            }
            ansi::CHA => {
                let target_col = (p0.max(1) as usize - 1).min(cols.saturating_sub(1));
                self.state.pos.1 = target_col;
            }
            ansi::SGR => self.handle_sgr(params),
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
