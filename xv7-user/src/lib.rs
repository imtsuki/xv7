#![no_std]
#![feature(llvm_asm)]
#![feature(lang_items)]

#[macro_use]
pub mod macros;

pub mod io;
pub mod process;
mod rt;

pub use usyscall as syscall;
