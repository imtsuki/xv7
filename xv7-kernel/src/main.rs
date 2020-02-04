#![no_std]
#![no_main]
#![feature(asm)]

mod console;
mod lang_item;
mod video;

#[macro_use]
extern crate embedded_graphics;

use embedded_graphics::{fonts::Font8x16, image::ImageBmp, pixelcolor::Rgb888, prelude::*};

#[inline(always)]
fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // `\x1B[2J` clears the screen, and `\x1B[H` moves the cursor to the home position.
    print!("\x1B[2J\x1B[H");
    println!("Now we are in kernel!");

    let mut display = video::GopDisplay;

    display.clear(RgbColor::WHITE);

    let logo: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("../resources/logo.bmp")).unwrap();

    logo.translate(
        (
            (display.size().width - logo.width()) as i32 / 2,
            (display.size().height - logo.height()) as i32 / 2,
        )
            .into(),
    )
    .draw(&mut display);

    egtext!(
        text = "XV7: Yet Another Operating System by imtsuki",
        top_left = (100, 100),
        style = text_style!(font = Font8x16, text_color = RgbColor::BLACK)
    )
    .draw(&mut display);

    hlt_loop();
}
