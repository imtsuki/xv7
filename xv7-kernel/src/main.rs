#![no_std]
#![no_main]
#![cfg_attr(doc, allow(unused_attributes))]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![feature(alloc_error_handler)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(llvm_asm)]
#![feature(naked_functions)]

extern crate alloc;

#[macro_use]
mod macros;

mod allocator;
pub mod ansi;
pub mod arch;
mod config;
pub mod device;
mod lang_item;
#[cfg(target_arch = "x86_64")]
mod memory;
mod pretty;
pub mod process;
pub mod syscall;
mod video;

pub fn kmain() -> ! {
    println!("Now we are in kernel!");
    arch::idle();
}
