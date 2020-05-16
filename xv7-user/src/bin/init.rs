//! user space init process (pid 0).
#![no_std]

#[macro_use]
extern crate xv7_user;

fn main() {
    println!("Hello from userspace!");
}
