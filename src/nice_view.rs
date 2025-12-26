use core::convert::Infallible;
use embassy_nrf::{gpio::Output, spim};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, Point, Size},
    primitives::Rectangle,
    Pixel,
};
use sharp_memory_display::*;

use embedded_hal::digital::v2::OutputPin;

struct NoPin;

impl OutputPin for NoPin {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct NiceView<'a> {
    parent: MemoryDisplay<spim::Spim<'a>, Output<'a>, NoPin>,
}

impl<'a> NiceView<'a> {
    pub fn new(spi: spim::Spim<'a>, cs: Output<'a>) -> Self {
        let disp = NoPin;
        Self {
            parent: MemoryDisplay::new(spi, cs, disp),
        }
    }

    /// Clear the screen and the internal framebuffer.
    pub fn clear(&mut self) {
        self.parent.clear()
    }

    /// Clear just the internal framebuffer, without writing changes to the display.
    pub fn clear_buffer(&mut self) {
        self.parent.clear_buffer()
    }

    /// Draw all lines of the buffer to the screen which have changed since last calling this function.
    pub fn flush_buffer(&mut self) {
        self.parent.flush_buffer()
    }
}

impl<'a> DrawTarget for NiceView<'a> {
    type Color = BinaryColor;
    type Error = spim::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let parent_width = self.parent.bounding_box().size.width as i32;

        self.parent.draw_iter(
            pixels
                .into_iter()
                .map(|Pixel(p, c)| Pixel(Point::new(parent_width - p.y, p.x), c)),
        )
    }
}

impl<'a> Dimensions for NiceView<'a> {
    fn bounding_box(&self) -> Rectangle {
        let parent_bb = self.parent.bounding_box();
        Rectangle::new(
            parent_bb.top_left,
            Size::new(parent_bb.size.height, parent_bb.size.width),
        )
    }
}
