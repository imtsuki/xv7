use crate::config::PAGE_OFFSET_BASE;
use boot::BootArgs;
use embedded_graphics::{
    image::Image,
    mono_font::{MonoTextStyle, iso_8859_1::FONT_8X13},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Baseline, Text},
};
use spin::Mutex;
use tinybmp::Bmp;

pub static GOP_DISPLAY: Mutex<Option<GopDisplay>> = Mutex::new(None);

pub struct GopDisplay(u64, (usize, usize));

impl OriginDimensions for GopDisplay {
    #[inline(always)]
    fn size(&self) -> Size {
        Size::new((self.1).0 as u32, (self.1).1 as u32)
    }
}

impl DrawTarget for GopDisplay {
    type Color = Rgb888;
    type Error = ();

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let Size { width, height } = self.size();
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.y >= 0 && (coord.x as u32) < width && (coord.y as u32) < height
            {
                let index = width as i32 * coord.y + coord.x;
                unsafe {
                    core::ptr::write_volatile(
                        ((PAGE_OFFSET_BASE + self.0) as *mut u32).add(index as usize),
                        Self::pack_rgb(color),
                    );
                }
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Rgb888) -> Result<(), ()> {
        let (width, height) = self.1;
        self.draw_solid_rect(0, 0, width, height, color);
        Ok(())
    }
}

impl GopDisplay {
    #[inline(always)]
    pub fn pack_rgb(color: Rgb888) -> u32 {
        ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | (color.b() as u32)
    }

    #[inline(always)]
    pub const fn resolution(&self) -> (usize, usize) {
        self.1
    }

    pub fn draw_solid_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Rgb888) {
        self.draw_solid_rect_packed(x, y, w, h, Self::pack_rgb(color));
    }

    pub fn draw_solid_rect_packed(&mut self, x: usize, y: usize, w: usize, h: usize, pixel: u32) {
        let (width, height) = self.1;
        if x >= width || y >= height || w == 0 || h == 0 {
            return;
        }
        let x_end = (x + w).min(width);
        let y_end = (y + h).min(height);
        let row_w = x_end - x;
        let base = (PAGE_OFFSET_BASE + self.0) as *mut u32;

        unsafe {
            if pixel == 0 && x == 0 && row_w == width {
                core::ptr::write_bytes(base.add(y * width), 0, (y_end - y) * width);
            } else if pixel == 0 {
                let mut ry = y;
                while ry < y_end {
                    core::ptr::write_bytes(base.add(ry * width + x), 0, row_w);
                    ry += 1;
                }
            } else {
                let mut ry = y;
                while ry < y_end {
                    let row = core::slice::from_raw_parts_mut(base.add(ry * width + x), row_w);
                    row.fill(pixel);
                    ry += 1;
                }
            }
        }
    }

    pub fn scroll_up_packed(&mut self, distance: usize, clear_width: usize, bg_px: u32) {
        let (width, height) = self.1;
        if distance == 0 || distance >= height {
            self.draw_solid_rect_packed(0, 0, width, height, bg_px);
            return;
        }
        let base = (PAGE_OFFSET_BASE + self.0) as *mut u32;
        unsafe {
            core::ptr::copy(
                base.add(distance * width),
                base,
                (height - distance) * width,
            );
        }
        let clear_w = clear_width.min(width);
        if clear_w > 0 {
            self.draw_solid_rect_packed(0, height - distance, clear_w, distance, bg_px);
        }
    }

    pub fn draw_glyph_8x16(
        &mut self,
        x: usize,
        y: usize,
        bitmap: &[u8; 13],
        fg_px: u32,
        bg_px: u32,
    ) {
        let (width, height) = self.1;
        if x + 8 > width || y + 16 > height {
            return;
        }
        let bg_row = [bg_px; 8];
        let base = (PAGE_OFFSET_BASE + self.0) as *mut u32;

        unsafe {
            let mut ry = 0;
            while ry < 13 {
                let bits = bitmap[ry];
                let dst = base.add((y + ry) * width + x) as *mut [u32; 8];
                if bits == 0 {
                    dst.write_volatile(bg_row);
                } else {
                    dst.write_volatile([
                        if (bits & 0x80) != 0 { fg_px } else { bg_px },
                        if (bits & 0x40) != 0 { fg_px } else { bg_px },
                        if (bits & 0x20) != 0 { fg_px } else { bg_px },
                        if (bits & 0x10) != 0 { fg_px } else { bg_px },
                        if (bits & 0x08) != 0 { fg_px } else { bg_px },
                        if (bits & 0x04) != 0 { fg_px } else { bg_px },
                        if (bits & 0x02) != 0 { fg_px } else { bg_px },
                        if (bits & 0x01) != 0 { fg_px } else { bg_px },
                    ]);
                }
                ry += 1;
            }
            while ry < 16 {
                let dst = base.add((y + ry) * width + x) as *mut [u32; 8];
                dst.write_volatile(bg_row);
                ry += 1;
            }
        }
    }
}

pub fn init(args: &BootArgs) {
    GOP_DISPLAY.lock().replace(GopDisplay(
        args.frame_buffer.base.as_u64(),
        args.frame_buffer.resolution,
    ));
}

#[allow(unused)]
pub fn splash_screen() {
    if let Some(display) = &mut *GOP_DISPLAY.lock() {
        display.clear(RgbColor::WHITE).unwrap();

        let img = Bmp::<Rgb888>::from_slice(include_bytes!("../resources/logo.bmp")).unwrap();
        let logo = Image::new(&img, Point::zero());

        logo.translate(
            (
                (display.size().width - img.size().width) as i32 / 2,
                (display.size().height - img.size().height) as i32 / 2,
            )
                .into(),
        )
        .draw(display)
        .unwrap();

        let style = MonoTextStyle::new(&FONT_8X13, RgbColor::BLACK);
        Text::with_baseline(
            "XV7: Yet Another Operating System by imtsuki",
            Point::new(100, 100),
            style,
            Baseline::Top,
        )
        .draw(display)
        .unwrap();
    }
}
