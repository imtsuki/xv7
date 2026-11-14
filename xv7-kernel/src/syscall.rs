pub mod fs;
pub mod process;

use alloc::string::String;
use core::str;

use abi::error::*;
use abi::number::*;

use crate::cpu::with_current_process;

const PATH_MAX: usize = 256;

pub fn syscall(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    fn inner(a: usize, b: usize, c: usize, d: usize, _e: usize, _f: usize) -> Result<usize> {
        match a {
            SYS_EXIT => process::exit(b as isize),
            SYS_WRITE => fs::write(b, c, d),
            SYS_READ => fs::read(b, c, d),
            SYS_FORK => process::fork(),
            SYS_GETPID => process::getpid(),
            SYS_YIELD => process::r#yield(),
            SYS_MKNOD => fs::mknod(&copy_user_path(b, c)?, d),
            SYS_OPEN => fs::open(&copy_user_path(b, c)?),
            _ => Err(Error::new(ENOSYS)),
        }
    }

    Error::mux(inner(a, b, c, d, e, f))
}

fn copy_user_path(user_ptr: usize, len: usize) -> Result<String> {
    if len == 0 {
        return Err(Error::new(ENOENT));
    }
    if len > PATH_MAX {
        return Err(Error::new(ENAMETOOLONG));
    }
    let mut buf = [0u8; PATH_MAX];
    with_current_process(|proc| proc.vm.copy_from_user(user_ptr, &mut buf[..len]))?;
    let s = str::from_utf8(&buf[..len]).map_err(|_| Error::new(EINVAL))?;
    Ok(String::from(s))
}
