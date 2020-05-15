//! user space init process (pid 0).

#![no_std]
#![no_main]

#[macro_use]
extern crate xv7_user;

#[no_mangle]
fn main() {
    println!("Hello from userspace!");
    panic!();
}
