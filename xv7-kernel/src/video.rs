use crate::config::PAGE_OFFSET_BASE;
use boot::BootArgs;
use embedded_graphics::{egtext, text_style};
use embedded_graphics::{fonts::Font8x16, image::Image, pixelcolor::Rgb888, prelude::*};
use tinybmp::Bmp;

pub struct GopDisplay(u64, (usize, usize));

impl DrawTarget<Rgb888> for GopDisplay {
    type Error = ();

    #[inline(always)]
    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) -> Result<(), ()> {
        let Pixel(coord, color) = pixel;
        let index = self.size().width as i32 * coord.y + coord.x;

        unsafe {
            core::ptr::write_volatile(
                ((PAGE_OFFSET_BASE + self.0) as *mut u32).add(index as usize),
                ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | (color.b() as u32),
            );
        }

        Ok(())
    }

    #[inline(always)]
    fn size(&self) -> Size {
        Size::new((self.1).0 as u32, (self.1).1 as u32)
    }
}

pub fn fun_things(args: &BootArgs) {
    let mut display = GopDisplay(
        args.frame_buffer.base.as_u64(),
        args.frame_buffer.resolution,
    );

    display.clear(RgbColor::WHITE).unwrap();

    let img = Bmp::from_slice(include_bytes!("../resources/logo.bmp")).unwrap();
    let logo = Image::new(&img, Point::zero());

    logo.translate(
        (
            (display.size().width - img.width()) as i32 / 2,
            (display.size().height - img.height()) as i32 / 2,
        )
            .into(),
    )
    .draw(&mut display)
    .unwrap();

    egtext!(
        text = "XV7: Yet Another Operating System by imtsuki",
        top_left = (100, 100),
        style = text_style!(font = Font8x16, text_color = RgbColor::BLACK)
    )
    .draw(&mut display)
    .unwrap();
}
