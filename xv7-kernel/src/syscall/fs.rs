use alloc::string::String;
use alloc::sync::Arc;

use abi::error::*;

use crate::cpu::with_current_process;
use crate::fs::FILE_SYSTEM;
use crate::fs::dev::Console;
use crate::fs::file::File;
use crate::memory::MapPerm;

const IO_CHUNK_SIZE: usize = 512;

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
        None => Err(Error::new(ENOENT)),
        Some(inode) => {
            let inode = inode.clone();
            let file = File::new(inode, true, true);
            with_current_process(|proc| {
                proc.fds.push(file);
                Ok(proc.fds.len() - 1)
            })
        }
    }
}

pub fn write(fd: usize, user_buf: usize, len: usize) -> Result<usize> {
    let (inode, mut offset, writable) = with_current_process(|proc| {
        proc.fds
            .get(fd)
            .map(|f| (f.inode(), f.offset(), f.is_writable()))
    })
    .ok_or_else(|| Error::new(EBADF))?;

    if !writable {
        return Err(Error::new(EBADF));
    }
    if len == 0 {
        return Ok(0);
    }

    with_current_process(|proc| {
        proc.vm
            .validate_user_range(user_buf, len, MapPerm::U | MapPerm::R)
    })?;

    let mut kbuf = [0u8; IO_CHUNK_SIZE];
    let mut total = 0usize;
    while total < len {
        let chunk = (len - total).min(IO_CHUNK_SIZE);
        with_current_process(|proc| proc.vm.copy_from_user(user_buf + total, &mut kbuf[..chunk]))?;
        let written = inode
            .write_at(offset, &kbuf[..chunk])
            .map_err(|_| Error::new(EIO))?;
        offset += written;
        total += written;
        if written < chunk {
            break;
        }
    }

    with_current_process(|proc| {
        if let Some(f) = proc.fds.get_mut(fd) {
            f.advance_offset(total);
        }
    });
    Ok(total)
}

pub fn read(fd: usize, user_buf: usize, len: usize) -> Result<usize> {
    let (inode, offset, readable) = with_current_process(|proc| {
        proc.fds
            .get(fd)
            .map(|f| (f.inode(), f.offset(), f.is_readable()))
    })
    .ok_or_else(|| Error::new(EBADF))?;

    if !readable {
        return Err(Error::new(EBADF));
    }
    if len == 0 {
        return Ok(0);
    }

    // Validate destination user buffer (including W permission) before blocking in `read_at`.
    with_current_process(|proc| {
        proc.vm
            .validate_user_range(user_buf, len, MapPerm::U | MapPerm::R | MapPerm::W)
    })?;

    let mut kbuf = [0u8; IO_CHUNK_SIZE];
    let chunk = len.min(IO_CHUNK_SIZE);
    let n = inode
        .read_at(offset, &mut kbuf[..chunk])
        .map_err(|_| Error::new(EIO))?;

    if n > 0 {
        with_current_process(|proc| proc.vm.copy_to_user(user_buf, &kbuf[..n]))?;
        with_current_process(|proc| {
            if let Some(f) = proc.fds.get_mut(fd) {
                f.advance_offset(n);
            }
        });
    }
    Ok(n)
}
