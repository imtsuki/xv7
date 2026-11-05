#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

#[macro_use]
pub mod macros;

pub mod arch;
pub mod io;
pub mod process;
mod rt;
pub mod syscall;

pub use abi::{Error, Result, error, number};
