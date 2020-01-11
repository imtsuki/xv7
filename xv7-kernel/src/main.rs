#![no_std]
#![no_main]
#![feature(asm)]

mod lang_item;

fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let buffer = 0x80000000 as *mut u32;

    for i in 0..(800 * 600) {
        unsafe {
            *buffer.offset(i) = 0xFFFFFFFF;
        }
    }

    hlt_loop();
}
