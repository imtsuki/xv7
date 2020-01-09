#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]

use core::panic::PanicInfo;

global_asm!(include_str!("boot.s"));

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
pub extern "C" fn bootloader_main() -> ! {
    hlt_loop!();
}
