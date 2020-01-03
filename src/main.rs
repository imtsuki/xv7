#![no_std]
#![no_main]
#![feature(global_asm)]

use core::panic::PanicInfo;

global_asm!(include_str!("boot.s"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn bootloader_main() -> ! {
    loop {}
}
