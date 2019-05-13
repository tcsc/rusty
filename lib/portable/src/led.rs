
use embedded_hal::{
    digital::v2::OutputPin
};

pub struct Led<PinT> {
    pin: PinT,
}

impl<T> Led<T> where T : OutputPin {
    pub fn new(pin: T) -> Led<T> {
        Led { pin }
    }

    pub fn on(&mut self) -> Result<(), T::Error> {
        self.pin.set_high()
    }

    pub fn off(&mut self) -> Result<(), T::Error> {
        self.pin.set_low()
    }
}