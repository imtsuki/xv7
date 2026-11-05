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
        while idx < buf.len() {
            match syscall::read(0, &mut buf[idx..]) {
                Ok(0) => break,
                Ok(n) => {
                    if let Some(newline_rel) = buf[idx..idx + n]
                        .iter()
                        .position(|&b| b == b'\n' || b == b'\r')
                    {
                        idx += newline_rel;
                        break;
                    }
                    idx += n;
                }
                Err(_) => panic!("read_line"),
            }
        }
        core::str::from_utf8(&buf[0..idx]).unwrap()
    }
}

pub fn stdin() -> Stdin {
    Stdin
}
