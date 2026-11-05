use alloc::string::String;
use alloc::sync::Arc;

use abi::error::*;

use crate::cpu::with_current_process;
use crate::fs::FILE_SYSTEM;
use crate::fs::dev::Console;
use crate::fs::file::File;

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
    match FILE_SYSTEM.lock().get(path) {
        None => dbg!(Err(Error::new(ENOENT))),
        Some(inode) => {
            let inode = inode.clone();
            let file = File::new(inode, true, true);
            with_current_process(|proc| {
                proc.fds.push(file);
                dbg!(Ok(proc.fds.len() - 1))
            })
        }
    }
}

pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    let (inode, offset, writable) = with_current_process(|proc| {
        proc.fds
            .get(fd)
            .map(|f| (f.inode(), f.offset(), f.is_writable()))
    })
    .ok_or_else(|| Error::new(ENOENT))?;

    if !writable {
        return Err(Error::new(EBADF));
    }

    let len = inode.write_at(offset, buf).map_err(|_| Error::new(EFAULT))?;
    with_current_process(|proc| {
        if let Some(f) = proc.fds.get_mut(fd) {
            f.advance_offset(len);
        }
    });
    Ok(len)
}

pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize> {
    let (inode, offset, readable) = with_current_process(|proc| {
        proc.fds
            .get(fd)
            .map(|f| (f.inode(), f.offset(), f.is_readable()))
    })
    .ok_or_else(|| Error::new(ENOENT))?;

    if !readable {
        return Err(Error::new(EBADF));
    }

    let len = inode.read_at(offset, buf).map_err(|_| Error::new(EFAULT))?;
    with_current_process(|proc| {
        if let Some(f) = proc.fds.get_mut(fd) {
            f.advance_offset(len);
        }
    });
    Ok(len)
}
