pub mod process;

use core::slice;
use core::str;

use usyscall::error::*;
use usyscall::number::*;

pub fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    fn inner(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> Result<usize> {
        match a {
            SYS_EXIT => process::exit(b as isize),
            SYS_WRITE => write(b, validate_slice(c as *const u8, d)?),
            _ => Err(Error::new(ENOSYS)),
        }
    }

    let result = inner(a, b, c, d, e, f);

    Error::mux(result)
}

pub fn validate_slice<T>(ptr: *const T, len: usize) -> Result<&'static [T]> {
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

pub fn write(fd: usize, buf: &[u8]) -> Result<usize> {
    if fd == 1 {
        let s = str::from_utf8(buf).map_err(|_| Error::new(EFAULT))?;
        print!("{}", s);
        Ok(buf.len())
    } else {
        Err(Error::new(ENOENT))
    }
}
