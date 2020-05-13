//! user space init process (pid 0).

#![no_std]
#![no_main]

extern crate xv7_user;

use xv7_user::syscall;

#[no_mangle]
fn main() {
    let mut code = 2;
    loop {
        code = syscall::hello(code);
    }
}
