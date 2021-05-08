use crate::syscall;
use core::fmt;
use core::fmt::Write;

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        syscall::write(1, s.as_bytes()).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}
pub struct Stdin;

impl Stdin {
    pub fn read_line<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let mut idx = 0;
        loop {
            if idx >= buf.len() {
                break;
            }
            if let Ok(n) = syscall::read(0, &mut buf[idx..idx + 1]) {
                if n > 0 {
                    if buf[idx] == b'\n' || buf[idx] == b'\r' {
                        break;
                    }
                    idx += n;
                }
            } else {
                panic!("read_line");
            }
        }
        core::str::from_utf8(&buf[0..idx]).unwrap()
    }
}

pub fn stdin() -> Stdin {
    Stdin
}
