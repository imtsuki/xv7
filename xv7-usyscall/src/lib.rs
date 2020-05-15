#![no_std]
#![feature(llvm_asm)]

pub mod arch;
pub mod error;
pub mod number;
pub mod syscall;

pub use error::Error;
pub use error::Result;

pub use syscall::*;
