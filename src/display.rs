//! Container to store and set display properties

use crate::command::Command;

use crate::properties::DisplayRotation;
use crate::properties::DisplaySize;

use display_interface::AsyncWriteOnlyDataCommand;
use display_interface::DataFormat;
use display_interface::DisplayError;

/// Display properties struct
pub struct Display<DI> {
    iface: DI,
    display_size: DisplaySize,
    display_rotation: DisplayRotation,
}

impl<DI> Display<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    /// Create new Display instance
    pub fn new(
        iface: DI,
        display_size: DisplaySize,
        display_rotation: DisplayRotation,
    ) -> Display<DI> {
        Display {
            iface,
            display_size,
            display_rotation,
        }
    }

    /// Release all resources used by the Display
    pub fn release(self) -> DI {
        self.iface
    }

    /// Initialise the display in column mode (i.e. a byte walks down a column of 8 pixels) with
    /// column 0 on the left and column _(display_width - 1)_ on the right.
    pub async fn init(&mut self) -> Result<(), DisplayError> {
        let (_, display_height) = self.display_size.dimensions();

        // TODO: Break up into nice bits so display modes can pick whathever they need
        Command::CommandLock(0x12).send(&mut self.iface).await?;
        Command::CommandLock(0xB1).send(&mut self.iface).await?;
        Command::DisplayOn(false).send(&mut self.iface).await?;
        Command::ClockDiv(0xF1).send(&mut self.iface).await?;
        Command::MuxRatio(display_height - 1)
            .send(&mut self.iface)
            .await?;
        Command::DisplayOffset(0).send(&mut self.iface).await?;
        Command::StartLine(0).send(&mut self.iface).await?;
        Command::SetGpio(0x00).send(&mut self.iface).await?;
        Command::FunctionSelect(0x01).send(&mut self.iface).await?;
        Command::SetVsl.send(&mut self.iface).await?;
        Command::Contrast(0x8F).send(&mut self.iface).await?;
        Command::ContrastCurrent(0x0F).send(&mut self.iface).await?;
        // Command::PhaseLength(0x32).send(&mut self.iface).await?;
        // Command::PreCharge(0x17).send(&mut self.iface).await?;
        Command::PreCharge(0x32).send(&mut self.iface).await?;
        Command::PreCharge2(0x01).send(&mut self.iface).await?;
        Command::Vcomh(0x05).send(&mut self.iface).await?;
        Command::Invert(false).send(&mut self.iface).await?;

        self.set_rotation(self.display_rotation).await.unwrap();

        self.clear().await?;

        Command::DisplayOn(true).send(&mut self.iface).await?;

        Ok(())
    }

    /// Clear the display by setting all pixels to black
    pub async fn clear(&mut self) -> Result<(), DisplayError> {
        let (display_width, display_height) = self.display_size.dimensions();
        self.set_draw_area((0, 0), (display_width, display_height))
            .await?;
        for _ in 0..(display_height as u32 * display_width as u32) {
            self.iface.send_data(DataFormat::U8(&[0x00, 0x00])).await?; // send 8 * 2 bits
        }
        Ok(())
    }

    /// Set the position in the framebuffer of the display where any sent data should be
    /// drawn. This method can be used for changing the affected area on the screen as well
    /// as (re-)setting the start point of the next `draw` call.
    pub async fn set_draw_area(
        &mut self,
        start: (u8, u8),
        end: (u8, u8),
    ) -> Result<(), DisplayError> {
        Command::Column(start.0, end.0.saturating_sub(1))
            .send(&mut self.iface)
            .await?;
        Command::Row(start.1, end.1.saturating_sub(1))
            .send(&mut self.iface)
            .await?;
        Command::WriteRam.send(&mut self.iface).await?;
        Ok(())
    }

    /// Send the data to the display for drawing at the current position in the framebuffer
    /// and advance the position accordingly. Cf. `set_draw_area` to modify the affected area by
    /// this method.
    pub async fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.iface.send_data(DataFormat::U8(buffer)).await?;
        Ok(())
    }

    /// Get the configured display size
    pub fn get_size(&self) -> DisplaySize {
        self.display_size
    }

    /// Get display dimensions, taking into account the current rotation of the display
    ///
    /// ```rust
    /// # struct FakeInterface;
    /// #
    /// # impl DisplayInterface for FakeInterface {
    /// #     fn send_command(&mut self, cmd: u8) -> Result<(), ()> { Ok(()) }
    /// #     fn send_data(&mut self, buf: &[u8]) -> Result<(), ()> { Ok(()) }
    /// # }
    /// #
    /// # let interface = FakeInterface {};
    /// #
    /// let disp = Display::new(
    ///     interface,
    ///     DisplaySize::Display128x128,
    ///     DisplayRotation::Rotate0
    /// );
    /// assert_eq!(disp.get_dimensions(), (128, 128));
    ///
    /// # let interface = FakeInterface {};
    /// let rotated_disp = Display::new(
    ///     interface,
    ///     DisplaySize::Display128x128,
    ///     DisplayRotation::Rotate90
    /// );
    /// assert_eq!(rotated_disp.get_dimensions(), (128, 128));
    /// ```
    pub fn get_dimensions(&self) -> (u8, u8) {
        let (w, h) = self.display_size.dimensions();

        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (w, h),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (h, w),
        }
    }

    /// Get the display rotation
    pub fn get_rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Set the display rotation
    pub async fn set_rotation(
        &mut self,
        display_rotation: DisplayRotation,
    ) -> Result<(), DisplayError> {
        self.display_rotation = display_rotation;

        match display_rotation {
            DisplayRotation::Rotate0 => {
                Command::SetRemap(false, false, true)
                    .send(&mut self.iface)
                    .await?;
            }
            DisplayRotation::Rotate90 => {
                Command::SetRemap(true, true, true)
                    .send(&mut self.iface)
                    .await?;
            }
            DisplayRotation::Rotate180 => {
                Command::SetRemap(false, true, false)
                    .send(&mut self.iface)
                    .await?;
            }
            DisplayRotation::Rotate270 => {
                Command::SetRemap(true, false, false)
                    .send(&mut self.iface)
                    .await?;
            }
        };

        Ok(())
    }
}
