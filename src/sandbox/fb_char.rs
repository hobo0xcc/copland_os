use crate::device::raspi3b::framebuffer::{FrameBuffer, FRAMEBUFFER};
use core::convert::Infallible;
use core::ops::{Deref, DerefMut};
use embedded_graphics::mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};

impl DrawTarget for FrameBuffer {
    type Color = Rgb888;
    type Error = Infallible;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let (x, y): (u32, u32) = coord.try_into().unwrap();
            if x < self.width && y < self.height {
                let index: usize = (y * self.width + x) as usize;
                unsafe {
                    self.fb.add(index).write(color.into_storage());
                }
            }
        }

        Ok(())
    }
}

impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

pub fn fb_char() {
    let display = unsafe { FRAMEBUFFER.deref_mut() };
    let character_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(0x00, 0x00, 0x00));
    Text::with_alignment(
        "Hello, world!",
        display.bounding_box().center(),
        character_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}
