use crate::arch::*;
use crate::number::*;
use crate::Result;
pub use crate::fs::*;

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

pub fn open(path: &str, flag: FileMode) -> Result<usize> {
    unsafe { syscall3(SYS_OPEN, path.as_ptr() as usize, path.len(), flag.bits() as usize) }
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

pub fn mkdir(path: &str, mode: usize) -> Result<usize> {
    unsafe { syscall3(SYS_MKDIR, path.as_ptr() as usize, path.len(), mode) }
}

pub fn mknod(path: &str, mode: usize) -> Result<usize> {
    unsafe { syscall3(SYS_MKNOD, path.as_ptr() as usize, path.len(), mode) }
}

pub fn stat(path: &str, stat: &mut Stat) -> Result<usize> {
    unsafe { syscall3(SYS_STAT, path.as_ptr() as usize, path.len(), stat as *mut Stat as usize) }
}

pub fn unlink(path: &str) -> Result<usize> {
    unsafe { syscall2(SYS_UNLINK, path.as_ptr() as usize, path.len()) }
}

pub fn getdents(fd: usize, dirs: &mut [Direntory]) -> Result<usize> {
    unsafe { syscall3(SYS_GETDENTS, fd, dirs.as_ptr() as usize, dirs.len()) }
}
