#![no_std]

mod audio;
pub mod cs43l22;

pub use audio::PeripheralClock;

use stm32f4xx_hal::stm32;

use cortex_m_semihosting::{hprintln};

fn pll_disable(rcc: &stm32::rcc::RegisterBlock) {
    rcc.cr.write(|dst| dst.plli2son().off());
    hprintln!("Waiting for I2S PLL to shut down").unwrap();
    while rcc.cr.read().plli2srdy().is_ready() {
        // TODO - add some sort of timeout
    }
    hprintln!("I2S PLL is shut down").unwrap()
}

fn pll_enable(rcc: &stm32::rcc::RegisterBlock) {
    rcc.cr.modify(|_, dst| dst.plli2son().on() );
    hprintln!("Waiting for I2S PLL to start up").unwrap();
    while rcc.cr.read().plli2srdy().is_not_ready() {
        // TODO - add some sort of timeout
    }
    hprintln!("I2S PLL to started").unwrap();
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