#![no_std]
#![no_main]
#![cfg_attr(doc, allow(unused_attributes))]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![feature(alloc_error_handler)]
#![feature(sync_unsafe_cell)]

extern crate alloc;

#[macro_use]
mod macros;

mod allocator;
pub mod ansi;
pub mod arch;
mod config;
pub mod cpu;
pub mod device;
pub mod fs;
#[cfg(target_arch = "x86_64")]
mod memory;
mod pretty;
pub mod process;
mod rt;
pub mod scheduler;
pub mod syscall;
mod video;

pub use crate::arch::context;
pub use crate::arch::paging;
pub use crate::arch::trapframe;

pub fn kmain() -> ! {
    println!("We are alive!");
    scheduler::enqueue(process::Process::initcode());
    scheduler::run();
}
