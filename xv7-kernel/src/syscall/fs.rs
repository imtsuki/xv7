use crate::fs::file::File;
use crate::fs::FILE_SYSTEM;
use crate::{fs::dev::Console, process::my_proc};
use alloc::string::String;
use alloc::sync::Arc;
use usyscall::error::*;

pub fn mknod(path: &str, dev: usize) -> Result<usize> {
    if dev == 1 {
        FILE_SYSTEM
            .lock()
            .insert(String::from(path), Arc::new(Console));
        Ok(0)
    } else {
        Err(Error::new(ENODEV))
    }
}

pub fn open(path: &str) -> Result<usize> {
    let proc = my_proc();
    match FILE_SYSTEM.lock().get(path) {
        None => dbg!(Err(Error::new(ENOENT))),
        Some(inode) => {
            let inode = inode.clone();
            let file = File::new(inode, true, true);
            proc.fds.push(file);
            dbg!(Ok(proc.fds.len() - 1))
        }
    }
}

pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    let proc = my_proc();

    match proc.fds.get_mut(fd) {
        Some(f) => f.write(buf).map_err(|_| Error::new(EFAULT)),
        None => Err(Error::new(ENOENT)),
    }
}

pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    let proc = my_proc();

    match proc.fds.get_mut(fd) {
        Some(f) => f.read(buf).map_err(|_| Error::new(EFAULT)),
        None => Err(Error::new(ENOENT)),
    }
}
