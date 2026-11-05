use crate::ansi;
use crate::ansi::C0;
use crate::device::console::Console;
use crate::video::GOP_DISPLAY;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, iso_8859_1::FONT_8X13},
    prelude::*,
    text::{Baseline, Text},
};
use vte::{Params, Parser, Perform};

struct Performer {
    pos: (usize, usize),
    size: (usize, usize),
}

pub struct MonitorConsole {
    parser: Parser,
    performer: Performer,
}

impl MonitorConsole {
    pub fn new() -> MonitorConsole {
        if let Some(display) = &mut *GOP_DISPLAY.lock() {
            let display_size = display.size();
            let (width, height) = (display_size.width as usize, display_size.height as usize);
            let rows = height / 16;
            let columns = width / 8;
            MonitorConsole {
                parser: Parser::new(),
                performer: Performer {
                    pos: (0, 0),
                    size: (rows, columns),
                },
            }
        } else {
            panic!("Display error");
        }
    }
}

impl Console for MonitorConsole {
    fn write(&mut self, buf: &[u8]) {
        self.parser.advance(&mut self.performer, buf);
    }
}

impl Performer {
    fn linefeed(&mut self) {
        let (mut line, _) = self.pos;
        line += 1;

        if line == self.size.0 {
            if let Some(display) = &mut *GOP_DISPLAY.lock() {
                display.scroll_up(16);
            }
            line = self.size.0 - 1;
        }
        self.pos = (line, 0)
    }

    fn backspace(&mut self) {
        let (line, col) = self.pos;
        self.pos = (line, if col > 0 { col - 1 } else { col });
        self.print(' ');
        let (line, col) = self.pos;
        self.pos = (line, if col > 0 { col - 1 } else { col });
    }

    fn next_char(&mut self) {
        let (mut line, mut col) = self.pos;
        col += 1;

        if col == self.size.1 {
            col = 0;
            line += 1;
        }

        if line == self.size.0 {
            if let Some(display) = &mut *GOP_DISPLAY.lock() {
                display.scroll_up(16);
            }
            line = self.size.0 - 1;
        }
        self.pos = (line, col)
    }
}

impl Perform for Performer {
    fn print(&mut self, c: char) {
        if let Some(display) = &mut *GOP_DISPLAY.lock() {
            let (line, col) = self.pos;

            let mut buf = [0; 4];
            let result = c.encode_utf8(&mut buf);
            let style = MonoTextStyleBuilder::new()
                .font(&FONT_8X13)
                .text_color(RgbColor::WHITE)
                .background_color(RgbColor::BLACK)
                .build();
            Text::with_baseline(
                result,
                Point::new(col as i32 * 8, line as i32 * 16),
                style,
                Baseline::Top,
            )
            .draw(display)
            .ok();
        }
        self.next_char();
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            C0::LF => self.linefeed(),
            C0::BS => self.backspace(),
            _ => (/* TODO */),
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        /* UNHANDLED */
    }

    fn put(&mut self, _byte: u8) {
        /* UNHANDLED */
    }

    fn unhook(&mut self) {
        /* UNHANDLED */
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        /* TODO */
    }

    fn csi_dispatch(
        &mut self,
        _params: &Params,
        intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if let Some(display) = &mut *GOP_DISPLAY.lock() {
            match (action, intermediates.first()) {
                (ansi::ED, None) => {
                    display.clear(RgbColor::BLACK).unwrap();
                }
                _ => (/* UNHANDLED */),
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        /* TODO */
    }
}
