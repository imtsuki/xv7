use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;

pub struct GopDisplay {}

impl DrawTarget<Rgb888> for GopDisplay {
    #[inline(always)]
    fn draw_pixel(&mut self, pixel: Pixel<Rgb888>) {
        let Pixel(coord, color) = pixel;
        let index = self.size().width as i32 * coord.y + coord.x;

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
