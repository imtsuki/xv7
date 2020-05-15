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
