use spin::Mutex;

use crate::arch::interrupt::{are_enabled, disable, enable_and_hlt, without_interrupts};
use crate::device::console::CONSOLE_DRIVERS;

const RAW_BUF_SIZE: usize = 1024;
const COOKED_BUF_SIZE: usize = 1024;
const LINE_BUF_SIZE: usize = 256;

struct RingBuf<const N: usize> {
    buf: [u8; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<const N: usize> RingBuf<N> {
    const fn new() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    #[inline]
    fn push(&mut self, byte: u8) -> bool {
        if self.len >= N {
            return false;
        }
        self.buf[self.tail] = byte;
        self.tail = (self.tail + 1) % N;
        self.len += 1;
        true
    }

    #[inline]
    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let byte = self.buf[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(byte)
    }
}

struct LineDiscipline {
    line_buf: [u8; LINE_BUF_SIZE],
    line_len: usize,
    cooked_rx: RingBuf<COOKED_BUF_SIZE>,
}

impl LineDiscipline {
    const fn new() -> Self {
        Self {
            line_buf: [0; LINE_BUF_SIZE],
            line_len: 0,
            cooked_rx: RingBuf::new(),
        }
    }

    /// Processes a single raw input byte and returns a static or inline echo slice.
    fn process_byte(&mut self, byte: u8, echo_out: &mut [u8; 4]) -> usize {
        match byte {
            b'\r' | b'\n' => {
                for i in 0..self.line_len {
                    self.cooked_rx.push(self.line_buf[i]);
                }
                self.cooked_rx.push(b'\n');
                self.line_len = 0;
                echo_out[0] = b'\n';
                1
            }
            b'\x08' | b'\x7f' => {
                if self.line_len > 0 {
                    // Pop one UTF-8 character (skip continuation bytes 0b10xx_xxxx)
                    while self.line_len > 0
                        && (self.line_buf[self.line_len - 1] & 0b1100_0000) == 0b1000_0000
                    {
                        self.line_len -= 1;
                    }
                    if self.line_len > 0 {
                        self.line_len -= 1;
                    }
                    echo_out[0] = b'\x08';
                    echo_out[1] = b' ';
                    echo_out[2] = b'\x08';
                    3
                } else {
                    0
                }
            }
            // Ctrl+U: erase entire current line
            0x15 => {
                let mut erased = false;
                if self.line_len > 0 {
                    self.line_len = 0;
                    erased = true;
                }
                let _ = erased;
                0
            }
            byte if byte == b'\t' || (0x20..0x7f).contains(&byte) || byte >= 0x80 => {
                if self.line_len < LINE_BUF_SIZE - 1 {
                    self.line_buf[self.line_len] = byte;
                    self.line_len += 1;
                    echo_out[0] = byte;
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

static RAW_RX: Mutex<RingBuf<RAW_BUF_SIZE>> = Mutex::new(RingBuf::new());
static TTY: Mutex<LineDiscipline> = Mutex::new(LineDiscipline::new());

/// Called from hardware interrupt handlers (`COM1`, PS/2 keyboard) to enqueue raw input bytes.
pub fn push_raw_byte(byte: u8) {
    without_interrupts(|| {
        RAW_RX.lock().push(byte);
        try_poll_and_echo();
    });
}

/// Non-blocking drain of `RAW_RX` through `LineDiscipline` using `try_lock`,
/// safe to invoke inside interrupt handlers for immediate character echo.
fn try_poll_and_echo() {
    let Some(mut raw) = RAW_RX.try_lock() else {
        return;
    };
    let Some(mut tty) = TTY.try_lock() else {
        return;
    };
    let Some(mut consoles) = CONSOLE_DRIVERS.try_lock() else {
        return;
    };

    let mut echo_buf = [0u8; 4];
    while let Some(byte) = raw.pop() {
        let echo_len = tty.process_byte(byte, &mut echo_buf);
        if echo_len > 0 {
            consoles.write(&echo_buf[..echo_len]);
        }
    }
}

/// Drains `RAW_RX` through `LineDiscipline` and echoes output to `CONSOLE_DRIVERS`.
pub fn poll_and_echo() {
    without_interrupts(|| {
        {
            let mut raw = RAW_RX.lock();
            while let Some(byte) = crate::arch::device::uart::try_receive() {
                raw.push(byte);
            }
        }
        let mut raw = RAW_RX.lock();
        let mut tty = TTY.lock();
        let mut consoles = CONSOLE_DRIVERS.lock();

        let mut echo_buf = [0u8; 4];
        while let Some(byte) = raw.pop() {
            let echo_len = tty.process_byte(byte, &mut echo_buf);
            if echo_len > 0 {
                consoles.write(&echo_buf[..echo_len]);
            }
        }
    });
}

/// Canonical blocking read from TTY into `buf`.
/// Waits for a newline-delimited line (using `hlt` between interrupts) and returns up to `buf.len()` bytes.
pub fn read(buf: &mut [u8]) -> usize {
    if buf.is_empty() {
        return 0;
    }

    loop {
        poll_and_echo();

        let cnt = without_interrupts(|| {
            let mut tty = TTY.lock();
            let mut count = 0;
            while count < buf.len() {
                let Some(byte) = tty.cooked_rx.pop() else {
                    break;
                };
                buf[count] = byte;
                count += 1;
                if byte == b'\n' {
                    break;
                }
            }
            count
        });

        if cnt > 0 {
            return cnt;
        }

        let was_enabled = are_enabled();
        enable_and_hlt();
        if !was_enabled {
            disable();
        }
    }
}
