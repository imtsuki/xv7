#![no_std]
#![no_main]
#![allow(unused_attributes)]
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

use boot::KernelArgs;

pub fn kmain(args: &KernelArgs) -> ! {
    println!("Now we are in kernel!");
    dbg!(args);
    #[cfg(target_arch = "x86_64")]
    memory::FRAME_ALLOCATOR.lock().hello();
    fun_things();
    arch::halt_loop();
}

#[macro_use]
extern crate embedded_graphics;

use embedded_graphics::{fonts::Font8x16, image::ImageBmp, pixelcolor::Rgb888, prelude::*};

fn fun_things() {
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
}
