//! user space init process.

#![no_std]
#![no_main]

extern crate xv7_user;

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    loop {}
}
