use core::arch::asm;

use crate::{Error, Result};

pub unsafe fn syscall0(mut a: usize) -> Result<usize> {
    asm!(
        "syscall",
        inlateout("rax") a,
        out("rcx") _,
        out("r11") _,
    );
    Error::demux(a)
}

pub unsafe fn syscall1(mut a: usize, b: usize) -> Result<usize> {
    asm!(
        "syscall",
        inlateout("rax") a,
        in("rdi") b,
        out("rcx") _,
        out("r11") _,
    );
    Error::demux(a)
}

pub unsafe fn syscall2(mut a: usize, b: usize, c: usize) -> Result<usize> {
    asm!(
        "syscall",
        inlateout("rax") a,
        in("rdi") b,
        in("rsi") c,
        out("rcx") _,
        out("r11") _,
    );
    Error::demux(a)
}

pub unsafe fn syscall3(mut a: usize, b: usize, c: usize, d: usize) -> Result<usize> {
    asm!(
        "syscall",
        inlateout("rax") a,
        in("rdi") b,
        in("rsi") c,
        in("rdx") d,
        out("rcx") _,
        out("r11") _,
    );
    Error::demux(a)
}

pub unsafe fn syscall4(mut a: usize, b: usize, c: usize, d: usize, e: usize) -> Result<usize> {
    asm!(
        "syscall",
        inlateout("rax") a,
        in("rdi") b,
        in("rsi") c,
        in("rdx") d,
        in("r10") e,
        out("rcx") _,
        out("r11") _,
    );
    Error::demux(a)
}

pub unsafe fn syscall5(
    mut a: usize,
    b: usize,
    c: usize,
    d: usize,
    e: usize,
    f: usize,
) -> Result<usize> {
    asm!(
        "syscall",
        inlateout("rax") a,
        in("rdi") b,
        in("rsi") c,
        in("rdx") d,
        in("r10") e,
        in("r8") f,
        out("rcx") _,
        out("r11") _,
    );
    Error::demux(a)
}
