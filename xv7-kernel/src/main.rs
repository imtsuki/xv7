#![no_std]
#![no_main]
#![feature(asm)]

mod lang_item;

#[macro_use]
extern crate embedded_graphics;

use embedded_graphics::image::ImageBmp;
use embedded_graphics::{
    drawable::Pixel,
    fonts::Font8x16,
    geometry::Size,
    pixelcolor::{Rgb888, RgbColor},
    prelude::*,
    DrawTarget,
};

struct GopDisplay {}

impl DrawTarget<Rgb888> for GopDisplay {
    #[inline(always)]
    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) {
        let Pixel(coord, color) = pixel;
        let index = 800 * coord.y + coord.x;

        unsafe {
            core::ptr::write_volatile(
                (0x80000000 as *mut u32).offset(index as isize),
                ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | (color.b() as u32),
            );
        }
    }

    #[inline(always)]
    fn size(&self) -> Size {
        Size::new(800, 600)
    }
}

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
    let mut display = GopDisplay {};

    display.clear(RgbColor::WHITE);

    egtext!(
        text = "XV7: Yet Another Operating System by imtsuki",
        top_left = (100, 100),
        style = text_style!(font = Font8x16, text_color = Rgb888::BLACK)
    )
    .draw(&mut display);

    let image: ImageBmp<Rgb888> = ImageBmp::new(include_bytes!("./img.bmp")).unwrap();

    let width = image.width() as i32;
    let height = image.height() as i32;

    image
        .translate((400 - width / 2, 300 - height / 2).into())
        .draw(&mut display);

    hlt_loop();
}
