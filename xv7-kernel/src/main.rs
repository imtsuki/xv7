#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

mod console;
mod gdt;
mod interrupt;
mod lang_item;
mod video;

#[macro_use]
extern crate embedded_graphics;

#[inline(always)]
fn halt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn kmain(args: &'static KernelArgs) -> ! {
    // Disable interrupts for safety.
    interrupt::without_interrupts(|| {
        // `\x1B[2J` clears the screen, and `\x1B[H` moves the cursor to the home position.
        print!("\x1B[2J\x1B[H");
        println!("Now we are in kernel!");

        println!("KernelArgs: {:#x?}", args);

        println!("IA32_APIC_BASE: {:#x}", unsafe {
            x86_64::registers::model_specific::Msr::new(0x1b).read()
        });

        gdt::init();
        interrupt::init();

        fun_things();
    });

    halt_loop();
}

use bootinfo::KernelArgs;
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
