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

pub fn open(path: &str) -> Result<usize> {
    unsafe { syscall2(SYS_OPEN, path.as_ptr() as usize, path.len()) }
}

pub fn close(fd: usize) -> Result<usize> {
    unsafe { syscall1(SYS_CLOSE, fd) }
}

pub fn exec(fd: usize, args: &[&str], envs: &[&str]) -> Result<usize> {
    unsafe {
        syscall5(
            SYS_EXEC,
            fd,
            args.as_ptr() as usize,
            args.len(),
            envs.as_ptr() as usize,
            envs.len(),
        )
    }
}

pub fn fork() -> Result<usize> {
    unsafe { syscall0(SYS_FORK) }
}

pub fn getpid() -> Result<usize> {
    unsafe { syscall0(SYS_GETPID) }
}

pub fn r#yield() -> Result<usize> {
    unsafe { syscall0(SYS_YIELD) }
}

pub fn mknod(path: &str, dev: usize) -> Result<usize> {
    unsafe { syscall3(SYS_MKNOD, path.as_ptr() as usize, path.len(), dev) }
}
