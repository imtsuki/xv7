pub mod fs;
pub mod process;

use core::slice;
use core::str;

use usyscall::error::*;
use usyscall::number::*;

pub fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    fn inner(a: usize, b: usize, c: usize, d: usize, _e: usize, _f: usize) -> Result<usize> {
        match a {
            SYS_EXIT => process::exit(b as isize),
            SYS_WRITE => fs::write(b, validate_slice(c as *const u8, d)?),
            SYS_READ => fs::read(b, validate_slice_mut(c as *mut u8, d)?),
            SYS_FORK => process::fork(),
            SYS_GETPID => process::getpid(),
            SYS_YIELD => process::r#yield(),
            SYS_MKNOD => fs::mknod(validate_str(b as *const u8, c)?, d),
            SYS_OPEN => fs::open(validate_str(b as *const u8, c)?),
            _ => Err(Error::new(ENOSYS)),
        }
    }

    let result = inner(a, b, c, d, e, f);

    Error::mux(result)
}

pub fn validate_slice<T>(ptr: *const T, len: usize) -> Result<&'static [T]> {
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

pub fn validate_slice_mut<T>(ptr: *mut T, len: usize) -> Result<&'static mut [T]> {
    Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
}

pub fn validate_str(ptr: *const u8, len: usize) -> Result<&'static str> {
    let slice = validate_slice(ptr, len)?;
    str::from_utf8(slice).map_err(|_| Error::new(EINVAL))
}
