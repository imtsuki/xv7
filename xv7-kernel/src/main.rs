#![no_std]
#![no_main]
#![cfg_attr(doc, allow(unused_attributes))]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![feature(asm)]

#[macro_use]
mod macros;

mod ansi;
mod arch;
mod config;
mod console;
mod lang_item;
#[cfg(target_arch = "x86_64")]
mod memory;
mod video;

pub fn kmain() -> ! {
    println!("Now we are in kernel!");

    #[cfg(target_arch = "x86_64")]
    memory::FRAME_ALLOCATOR.lock().hello();
    video::fun_things();
    arch::idle();
}
