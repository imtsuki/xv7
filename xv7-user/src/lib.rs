#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

#[macro_use]
pub mod macros;

pub mod io;
pub mod process;
mod rt;

pub use usyscall as syscall;
