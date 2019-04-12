
pub struct Led<PinT> {
    pin: PinT,
}

impl<T> Led<T> where T : ::embedded_hal::digital::OutputPin {
    pub fn new(pin: T) -> Led<T> {
        Led { pin }
    } 

    pub fn on(&mut self) {
        self.pin.set_high();
    }

    pub fn off(&mut self) {
        self.pin.set_low();
    }
}