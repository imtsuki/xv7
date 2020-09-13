pub use crate::fs::*;
use crate::syscall;
pub use crate::syscall::*;

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

pub fn write(fd: usize, buf: &[u8]) -> usize {
    syscall::write(fd, buf).unwrap()
}

pub fn read(fd: usize, buf: &mut [u8]) -> usize {
    syscall::read(fd, buf).unwrap()
}

pub fn open(path: &str, flag: FileMode) -> usize {
    syscall::open(path, flag).unwrap()
}

pub fn close(fd: usize) -> usize {
    syscall::close(fd).unwrap()
}

pub fn mkdir(path: &str, mode: usize) -> usize {
    syscall::mkdir(path, mode).unwrap()
}

pub fn mknod(path: &str, mode: usize) -> usize {
    syscall::mknod(path, mode).unwrap()
}

pub fn stat(path: &str, stat: &mut Stat) -> usize {
    syscall::stat(path, stat).unwrap()
}

pub fn unlink(path: &str) -> usize {
    syscall::unlink(path).unwrap()
}

pub fn getdents(fd: usize, dirs: &mut [Direntory]) -> usize {
    syscall::getdents(fd, dirs).unwrap()
}
