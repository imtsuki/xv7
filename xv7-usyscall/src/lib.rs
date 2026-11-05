#![no_std]

pub mod arch;
pub mod error;
pub mod number;
pub mod syscall;

pub use error::Error;
pub use error::Result;

pub use syscall::*;
