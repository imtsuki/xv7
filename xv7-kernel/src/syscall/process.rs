use abi::error::*;

use crate::cpu::{self, with_current_process};

pub fn exit(code: isize) -> Result<usize> {
    with_current_process(|proc| {
        if proc.pid == 0 {
            panic!("pid 0 exited with code {}", code);
        }
        Ok(0)
    })
}

pub fn exec(path: &str) {
    with_current_process(|proc| {
        proc.exec(path);
    });
}

pub fn fork() -> Result<usize> {
    with_current_process(|_proc| Ok(0))
}

pub fn getpid() -> Result<usize> {
    with_current_process(|proc| Ok(proc.pid))
}

pub(crate) fn r#yield() -> Result<usize> {
    cpu::yield_current_process();
    Ok(0)
}
