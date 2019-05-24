#![no_std]

mod audio;
pub mod cs43l22;
use log::info;

pub use audio::PeripheralClock;

use stm32f4xx_hal::stm32;

fn pll_disable(rcc: &stm32::rcc::RegisterBlock) {
    rcc.cr.write(|dst| dst.plli2son().off());
    info!("Waiting for I2S PLL to shut down");
    while rcc.cr.read().plli2srdy().is_ready() {
        // TODO - add some sort of timeout
    }
    info!("I2S PLL is shut down");
}

fn pll_enable(rcc: &stm32::rcc::RegisterBlock) {
    rcc.cr.modify(|_, dst| dst.plli2son().on() );
    info!("Waiting for I2S PLL to start up");
    while rcc.cr.read().plli2srdy().is_not_ready() {
        // TODO - add some sort of timeout
    }
    info!("I2S PLL to started");
}


pub fn set_i2s_clock(rcc: &stm32::rcc::RegisterBlock,
                     multiplier: u16,
                     divisor: u8) {
    pll_disable(rcc);
    rcc.plli2scfgr.modify(|_, w| unsafe {
        w.plli2sn().bits(multiplier)
            .plli2sr().bits(divisor)
    });
    pll_enable(rcc);
}