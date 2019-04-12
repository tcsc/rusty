
#[repr(u32)]
pub enum PeripheralClock {
    I2S = 0x0000_0001,
    RealTime = 0x0000_0002,
    PLLI2S = 0x0000_0004
} 