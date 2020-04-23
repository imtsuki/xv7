#![no_std]
#![no_main]
#![cfg_attr(doc, allow(unused_attributes))]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![feature(alloc_error_handler)]
#![feature(llvm_asm)]
#![feature(box_syntax)]
#![feature(box_patterns)]

extern crate alloc;

#[macro_use]
mod macros;

mod allocator;
mod ansi;
pub mod arch;
mod config;
mod console;
mod lang_item;
#[cfg(target_arch = "x86_64")]
mod memory;
mod pretty;
mod video;

pub fn kmain() -> ! {
    println!("Now we are in kernel!");
    arch::idle();
}
