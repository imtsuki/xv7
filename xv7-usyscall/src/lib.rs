#![no_std]
#![feature(llvm_asm)]

#[macro_use]
extern crate bitflags;

pub mod arch;
pub mod error;
pub mod fs;
pub mod number;
pub mod syscall;

pub use error::Error;
pub use error::Result;

pub use syscall::*;
