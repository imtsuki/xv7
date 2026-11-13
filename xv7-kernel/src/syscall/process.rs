use abi::error::*;

use crate::cpu::with_current_process;
use crate::scheduler;

pub fn exit(code: isize) -> Result<usize> {
    scheduler::exit_current(code);
}

pub fn fork() -> Result<usize> {
    Err(Error::new(ENOSYS))
}

pub fn getpid() -> Result<usize> {
    with_current_process(|proc| Ok(proc.pid))
}

pub(crate) fn r#yield() -> Result<usize> {
    scheduler::yield_now();
    Ok(0)
}
