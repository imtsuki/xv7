use crate::cpu;
use crate::fs;
use core::str;
use usyscall::error::*;
use usyscall::fs::*;

pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    // TODO: implement devfs
    if fd < 3 {
        let s = str::from_utf8(buf).map_err(|_| Error::new(EFAULT))?;
        print!("{}", s);
        Ok(buf.len())
    } else {
        let file = cpu::my_cpu().current_process.as_ref().unwrap().fdt.get_file(fd)?;
        fs::REGISTERED_FS.lock().vfs_write(&file, buf)
    }
}

pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    // TODO: implement devfs
    if fd < 3 {
        todo!()
    }else{
        let file = cpu::my_cpu().current_process.as_ref().unwrap().fdt.get_file(fd)?;
        fs::REGISTERED_FS.lock().vfs_read(&file, buf)
    }
}

pub fn open(path: &str, flag: FileMode) -> Result<usize> {
    let file = fs::REGISTERED_FS.lock().vfs_open(path, flag)?;
    cpu::my_cpu().current_process.as_mut().unwrap().fdt.open_fd(&file)
}

pub fn close(fd: usize) -> Result<usize> {
    cpu::my_cpu().current_process.as_mut().unwrap().fdt.close_fd(fd)
}

pub fn mkdir(path: &str, mode: usize) -> Result<usize> {
    fs::REGISTERED_FS.lock().vfs_mkdir(path).map(|_| 0usize)
}

pub fn mknod(path: &str, mode: usize) -> Result<usize> {
    fs::REGISTERED_FS.lock().vfs_create(path).map(|_| 0usize)
}

pub fn stat(path: &str, stat: &mut Stat) -> Result<usize> {
    fs::REGISTERED_FS
        .lock()
        .vfs_stat(path, stat)
        .map(|_| 0usize)
}

pub fn unlink(path: &str) -> Result<usize> {
    fs::REGISTERED_FS.lock().vfs_unlink(path).map(|_| 0usize)
}

pub fn getdents(fd: usize, dirs: &mut [Direntory]) -> Result<usize> {
    let file = cpu::my_cpu().current_process.as_ref().unwrap().fdt.get_file(fd)?;
    fs::REGISTERED_FS.lock().vfs_readdir(&file, dirs)
}
