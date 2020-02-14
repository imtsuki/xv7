#![no_std]
#![no_main]
#![feature(asm)]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![allow(unused_attributes)]

#[macro_use]
extern crate embedded_graphics;
#[macro_use]
mod macros;

mod ansi;
mod config;
mod console;
mod lang_item;
mod memory;
mod video;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

use boot::KernelArgs;

pub fn kmain(args: &KernelArgs) -> ! {
    println!("Now we are in kernel!");
    dbg!(args);
    memory::FRAME_ALLOCATOR.lock().hello();
    fun_things();
    arch::halt_loop();
}

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
