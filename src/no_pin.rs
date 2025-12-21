use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;

pub struct NoPin;

impl OutputPin for NoPin {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
