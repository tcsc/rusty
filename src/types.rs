
use stm32f407_audio::cs43l22;

use stm32f4xx_hal::{
    stm32,
    gpio::{
        self, Input, Output, PullDown, PushPull,
        gpioa::{PA0, PA9, PA10},
        gpiob::{PB6, PB9},
        gpiod::{PD4},
        AF4, AF7},
    i2c::I2c,
    serial::Serial
};

pub type UserButtonPin = PA0<Input<PullDown>>;
type I2C1SclPin =  PB6<gpio::Alternate<AF4>>;
type I2C1SdaPin = PB9<gpio::Alternate<AF4>>;
pub type I2CBus1 = I2c<stm32::I2C1, (I2C1SclPin, I2C1SdaPin)>;
pub type AudioResetPin = PD4<Output<PushPull>>;
pub type Cs43l22 = cs43l22::Driver<I2CBus1, AudioResetPin>;

pub type Usart1Tx = PA9<gpio::Alternate<AF7>>;
pub type Usart1Rx = PA10<gpio::Alternate<AF7>>;
pub type Usart1 = Serial<stm32::USART1, (Usart1Tx, Usart1Rx)>;