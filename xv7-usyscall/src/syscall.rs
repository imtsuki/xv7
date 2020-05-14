use crate::arch::*;
use crate::number::*;
use crate::Result;

pub fn exit(code: isize) -> ! {
    unsafe {
        syscall1(SYS_EXIT, code as usize).unwrap();
    }
    unreachable!()
}

pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len()) }
}

pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    unsafe { syscall3(SYS_READ, fd, buf.as_mut_ptr() as usize, buf.len()) }
}
