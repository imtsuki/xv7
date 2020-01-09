#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;

macro_rules! hlt_loop {
    () => {
        loop {
            unsafe {
                asm!("hlt");
            }
        }
    };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop!();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    hlt_loop!();
}
