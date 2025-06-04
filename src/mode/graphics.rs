use crate::display::Display;
use display_interface::{AsyncWriteOnlyDataCommand, DisplayError};
use hal::delay::DelayNs;
use hal::digital::OutputPin;
use shared_display_core::{CompressableDisplay, SharableBufferedDisplay};

use crate::mode::displaymode::DisplayModeTrait;
use crate::properties::DisplayRotation;

extern crate alloc;
use alloc::vec::Vec;

/// Graphics Mode for the display
pub struct GraphicsMode<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    display: Display<DI>,
    #[cfg(feature = "buffered")]
    pub buffer: &'static mut [u8],
}

impl<DI> DisplayModeTrait<DI> for GraphicsMode<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    #[cfg(not(feature = "buffered"))]
    /// Create new GraphicsMode instance
    fn new(display: Display<DI>) -> Self {
        GraphicsMode { display }
    }

    #[cfg(feature = "buffered")]
    fn new(display: Display<DI>, buffer: &'static mut [u8]) -> Self {
        GraphicsMode { display, buffer }
    }

    #[cfg(not(feature = "buffered"))]
    /// Release all resources used by GraphicsMode
    fn release(self) -> Display<DI> {
        self.display
    }

    #[cfg(feature = "buffered")]
    /// Release all resources used by GraphicsMode
    fn release(self) -> (Display<DI>, &'static mut [u8]) {
        (self.display, self.buffer)
    }
}

// impl<DI: DisplayInterface> GraphicsMode<DI> {
//     /// Create a new grahpics display interface
//     pub fn new(display: Display<DI>) -> Self {
//         GraphicsMode { display }
//     }
// }

impl<DI> GraphicsMode<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    #[cfg(not(feature = "buffered"))]
    /// Clear the display
    pub async fn clear(&mut self) {
        self.display.clear().await.unwrap();
    }

    #[cfg(feature = "buffered")]
    /// Clear the display
    pub async fn clear(&mut self, flush: bool) {
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0u8;
        }
        if flush {
            self.flush().await;
        }
    }

    /// Reset display
    pub fn reset<RST, DELAY>(&mut self, rst: &mut RST, delay: &mut DELAY) -> Result<(), RST::Error>
    where
        RST: OutputPin,
        DELAY: DelayNs,
    {
        rst.set_high()?;
        delay.delay_ms(1);
        rst.set_low()?;
        delay.delay_ms(10);
        rst.set_high()?;
        Ok(())
    }

    #[cfg(feature = "buffered")]
    /// Access the framebuffer
    pub fn fb(&self) -> &[u8] {
        self.buffer
    }

    #[cfg(feature = "buffered")]
    /// Access the framebuffer
    pub fn fb_mut(&mut self) -> &mut [u8] {
        self.buffer
    }

    #[cfg(not(feature = "buffered"))]
    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub async fn set_pixel(&mut self, x: u32, y: u32, color: u16) {
        let (display_width, display_height) = self.display.get_size().dimensions();
        let rot = self.display.get_rotation();
        let (nx, ny) = match rot {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (x, y),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (y, x),
        };
        self.display
            .set_draw_area((nx as u8, ny as u8), (display_width, display_height))
            .await
            .unwrap();
        self.display
            .draw(&[(color >> 8) as u8, color as u8])
            .await
            .unwrap();
    }

    #[cfg(feature = "buffered")]
    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub async fn set_pixel(&mut self, x: u32, y: u32, color: u16) {
        // set bytes in buffer
        self.buffer[(y as usize * 128usize + x as usize) * 2] = (color >> 8) as u8;
        self.buffer[((y as usize * 128usize + x as usize) * 2) + 1usize] = color as u8;
    }

    #[cfg(feature = "buffered")]
    pub async fn flush(&mut self) {
        let (display_width, display_height) = self.display.get_size().dimensions();
        self.display
            .set_draw_area((0, 0), (display_width, display_height))
            .await
            .unwrap();
        self.display.draw(self.buffer).await.unwrap();
    }

    #[cfg(feature = "buffered")]
    pub async fn flush_area(&mut self, area: &Rectangle) {
        let (display_width, _) = self.display.get_size().dimensions();
        let num_rows = area.size.height;
        let bottom_right = area.bottom_right().unwrap_or(area.top_left);

        // flush row by row
        for row in 0..num_rows {
            let row_index = area.top_left.y as usize + row as usize;
            self.display
                .set_draw_area(
                    (area.top_left.x as u8, row_index as u8),
                    (bottom_right.x as u8 + 1, row_index as u8 + 1),
                )
                .await
                .unwrap();

            let row_start = (row_index * display_width as usize + area.top_left.x as usize) * 2;
            let row_in_buffer = &self.buffer[row_start..(row_start + 2 * area.size.width as usize)];
            self.display.draw(row_in_buffer).await.unwrap();
        }
    }

    /// Display is set up in column mode, i.e. a byte walks down a column of 8 pixels from
    /// column 0 on the left, to column _n_ on the right
    pub async fn init(&mut self) -> Result<(), DisplayError> {
        self.display.init().await?;
        Ok(())
    }

    /// Set the display rotation
    pub async fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.display.set_rotation(rot).await
    }

    /// Get display dimensions, taking into account the current rotation of the display
    pub fn get_dimensions(&self) -> (u8, u8) {
        self.display.get_dimensions()
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics_core;
#[cfg(feature = "graphics")]
use self::embedded_graphics_core::prelude::{
    Dimensions, DrawTarget, OriginDimensions, Pixel, RawData, Size,
};
#[cfg(feature = "graphics")]
use self::embedded_graphics_core::{
    pixelcolor::{raw::RawU16, Rgb565},
    primitives::Rectangle,
};
#[cfg(all(feature = "graphics", not(feature = "buffered")))]
use self::embedded_graphics_core::{prelude::PointsIter, primitives::Rectangle};

#[cfg(feature = "graphics")]
impl<DI: AsyncWriteOnlyDataCommand> DrawTarget for GraphicsMode<DI> {
    type Color = Rgb565;
    type Error = ();

    async fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        let contained_pixels = pixels.into_iter().filter(|Pixel(pos, _)| bb.contains(*pos));
        for Pixel(pos, color) in contained_pixels {
            self.set_pixel(pos.x as u32, pos.y as u32, RawU16::from(color).into_inner())
                .await
        }
        Ok(())
    }

    #[cfg(not(feature = "buffered"))]
    async fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());

        let rot = self.display.get_rotation();
        let sx = drawable_area.top_left.x as u8;
        let sy = drawable_area.top_left.y as u8;
        let ex = (drawable_area.top_left.x as u32 + drawable_area.size.width) as u8;
        let ey = (drawable_area.top_left.y as u32 + drawable_area.size.height) as u8;

        // Set the draw area to the size of the rectangle
        let (area_start, area_end) = match rot {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => ((sx, sy), (ex, ey)),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => ((sy, sx), (ey, ex)),
        };

        self.display
            .set_draw_area(area_start, area_end)
            .await
            .unwrap();

        // Get an iterator of colours as u16
        // Check points for containment
        let iter = area
            .points()
            .zip(colors)
            .filter(|(pos, _)| drawable_area.contains(*pos))
            .map(|(_, color)| RawU16::from(color).into_inner());
        for color in iter {
            self.display
                .draw(&[(color >> 8) as u8, color as u8])
                .await
                .unwrap()
        }

        Ok(())
    }
}

impl<DI: AsyncWriteOnlyDataCommand> OriginDimensions for GraphicsMode<DI> {
    fn size(&self) -> Size {
        let dim = self.display.get_size().dimensions();
        Size::from((dim.0 as u32, dim.1 as u32))
    }
}

#[cfg(feature = "buffered")]
impl<DI: AsyncWriteOnlyDataCommand> SharableBufferedDisplay for GraphicsMode<DI> {
    type BufferElement = u16;

    fn get_buffer(&mut self) -> &mut [Self::BufferElement] {
        assert_eq!(self.buffer.len() % 2, 0);
        let buffer_len_in_u16s = self.buffer.len() / 2;
        let ptr = self.buffer.as_mut_ptr() as *mut u16;
        assert_eq!(
            ptr.align_offset(::core::mem::align_of::<u16>()),
            0,
            "Misaligned pointer for u16"
        );

        unsafe { ::core::slice::from_raw_parts_mut(ptr, buffer_len_in_u16s) }
    }

    fn calculate_buffer_index(
        point: embedded_graphics_core::prelude::Point,
        buffer_area_size: Size,
    ) -> usize {
        point.y as usize * buffer_area_size.width as usize + point.x as usize
    }

    fn map_to_buffer_element(color: Self::Color) -> Self::BufferElement {
        RawU16::from(color).into_inner()
    }
}

#[cfg(all(feature = "buffered", feature = "graphics"))]
impl<DI: AsyncWriteOnlyDataCommand> CompressableDisplay for GraphicsMode<DI> {
    async fn flush_chunk(&mut self, chunk: Vec<Self::BufferElement>, chunk_area: Rectangle) {
        let bottom_right = chunk_area.bottom_right().unwrap_or(chunk_area.top_left);
        self.display
            .set_draw_area(
                (chunk_area.top_left.x as u8, chunk_area.top_left.y as u8),
                (bottom_right.x as u8 + 1, bottom_right.y as u8 + 1),
            )
            .await
            .expect("set_draw_area for entire chunk failed");
        let chunk_as_bytes: &[u8] = unsafe {
            core::slice::from_raw_parts(
                chunk.as_ptr() as *const u8,
                chunk.len() * core::mem::size_of::<u16>(),
            )
        };
        self.display.draw(chunk_as_bytes).await.unwrap();
    }

    fn drop_buffer(&mut self) {
        // this does not actually de-allocate the buffer!
        self.buffer = &mut [];
    }
}
