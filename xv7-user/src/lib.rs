#![no_std]
#![feature(llvm_asm)]

#[macro_use]
pub mod macros;

mod rt;
pub use usyscall::*;
