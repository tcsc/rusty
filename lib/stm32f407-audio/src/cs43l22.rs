

pub struct Driver<ResetPinT> {
    reset_pin: ResetPinT
}

impl<ResetPinT> Driver<ResetPinT> 
    where ResetPinT : ::embedded_hal::digital::OutputPin
{
    pub fn new(reset_pin: ResetPinT) -> Driver<ResetPinT> {
        Driver { reset_pin }
    }
}