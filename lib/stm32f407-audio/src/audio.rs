
use stm32f4xx_hal::time::Hertz;

pub struct I2S;
pub struct RTC;
pub struct PLLI2S;

pub struct ClockConfig<CLOCK> {
    // clock config structure from HAL

    // type discrimnator
    clock_device: CLOCK
}

pub enum ClockError {
    BadValue
}

impl ClockConfig<I2S> {
    pub fn init<F: Into<Hertz>>(numerator: F, denominator: u32) -> Result<(), ClockError> {
        
        // disable I2S pll
        // get system tick 
        // wait on I2S is reporting disabled (with timeout)
        // set PLL division facors
        // enable I2S PLL
        // Wait until I2S is reporting "on"
        // 

        Ok (())
    }
}

#[repr(u32)]
pub enum PeripheralClock {
    I2S = 0x00000001,
    RealTime = 0x00000002,
    PLLI2S = 0x00000004
} 

pub fn set_peripheral_clock(clock: PeripheralClock) {
                            
}


