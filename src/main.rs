#![no_std]
#![no_main]

// RTFM macros need this for clean clippy report
#![allow(clippy::toplevel_ref_arg)]

extern crate panic_semihosting;

use cortex_m_semihosting::hprintln;

use stm32f407_audio::{self as audio /*, cs43l22 */};
use serial_log::SerialLogger;
use log::{info, Level, LevelFilter};
use nb;
mod types;
use types::*;
use stm32f4xx_hal::serial::Serial;

use rtfm::{app};//, Instant};
use heapless::consts::U256;

use stm32f4xx_hal::{
    prelude::*,
    stm32::{self, RCC, USART2},
    gpio::{self, Input, Output, PushPull, Floating,
        gpiob::{PB6, PB9},
        gpiod::{PD4, PD15},
    },
    i2c::I2c,
    rcc::Clocks,
    serial::{
        //PinTx, PinRx, Serial,
        config::{
            Config as SerialConfig,
            Parity,
            StopBits,
            WordLength,
        }
    }
};

use portable::{Button, ButtonEvent, Led};

const GPIO_POLL_INTERVAL : u32 = 840_000;
const AUDIO_DEVICE_ADDR : u8 = 0x4a;

// A lazily-initialised global logger. 
static mut LOGGER : Option<SerialLogger<LogUart, U256>> = None;

fn init_audio(
    i2c1: stm32::I2C1,
    i2c_scl: PB6<Input<Floating>>,
    i2c_sda: PB9<Input<Floating>>,
    audio_reset: PD4<Input<Floating>>,
    clocks: Clocks)
        -> Cs43l22
{
    let scl = i2c_scl.into_alternate_af4()
                 .set_speed(gpio::Speed::High)
                 .set_open_drain();

    let sda = i2c_sda.into_alternate_af4()
                 .set_speed(gpio::Speed::High)
                 .set_open_drain();

    let audio_reset = audio_reset.into_push_pull_output()
                                 .set_speed(gpio::Speed::High);

    let audio_i2c = I2c::i2c1(i2c1, (scl, sda), 100.khz(), clocks);

    audio::cs43l22::Driver::init(audio_i2c,
        AUDIO_DEVICE_ADDR,
        audio_reset).unwrap()
}

fn init_logger(uart: USART2,
               tx_pin: LogTxPin,
               rx_pin: LogRxPin,
               clocks: Clocks) {
    let config = SerialConfig {
        baudrate: 19200.bps(),
        wordlength: WordLength::DataBits8,
        parity: Parity::ParityNone,
        stopbits: StopBits::STOP1
    };

    let mut serial_port = Serial::usart2(
        uart, (tx_pin, rx_pin), config, clocks).unwrap();

    for b in "\r\nWelcome to rusty!\r\n".as_bytes() {
        nb::block!(serial_port.write(*b)).is_ok();
    }

    let logger = SerialLogger::new(serial_port, Level::Debug);
    let logref = unsafe {
        crate::LOGGER = Some(logger);
        crate::LOGGER.as_ref().unwrap()
    };

    log::set_logger(logref)
        .map(|_| log::set_max_level(LevelFilter::Trace))
        .unwrap();
}

#[app(device = stm32f4xx_hal::stm32)]
const APP: () = {
    static mut USER_BUTTON : Button<UserButtonPin> = ();
    static mut LED_BLUE : Led<PD15<Output<PushPull>>> = ();
    static mut AUDIO_DEVICE : Cs43l22 = ();

    #[init()]
    fn init() {
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.mhz())
                             .sysclk(168.mhz())
                             .freeze();

        // enable GPIOx port clocks
        let rcc_registers = unsafe { &*RCC::ptr() };
        rcc_registers.ahb1enr.write(|w| {
            w.gpioaen().bit(true) // user button
             .gpioben().bit(true) // I2C bus
             .gpiocen().bit(true)
             .gpioden().bit(true) // LEDs
        });

        // initialise logging over UART1
        let gpio_port_a = device.GPIOA.split();
        let gpio_port_b = device.GPIOB.split();

        init_logger(device.USART2,
                    gpio_port_a.pa2.into_alternate_af7(),
                    gpio_port_a.pa3.into_alternate_af7(),
                    clocks);

        info!("Setting I2S clock");
        // prepare I2S PLL clock for 44.1 kHz output
        audio::set_i2s_clock(rcc_registers, 290, 2);
        info!("I2S clock set");

        // enable I2S clocks
        rcc_registers.apb1enr.write(|w| {
            w.i2c1en().bit(true)
             //.spi3en().bit(true)
             //.dma1en().bit(true)
        });

        // configure GPIO pin for user button
        let user_button_pin = gpio_port_a.pa0
                                         .into_pull_down_input();

        // // schedule the GPIO polling task
        // schedule.poll_gpio(Instant::now()).unwrap();

        let gpio_port_d = device.GPIOD.split();
        let audio_device = init_audio(device.I2C1,
                   gpio_port_b.pb6,
                   gpio_port_b.pb9,
                   gpio_port_d.pd4,
                   clocks);

        // Add all of the resources
        // DELAY = Delay::new(core.SYST, clocks);
        USER_BUTTON = Button::new(user_button_pin);
        LED_BLUE = Led::new(gpio_port_d.pd15
                                       .into_push_pull_output());
        AUDIO_DEVICE = audio_device;
    }

    // //#[task(resources = [USER_BUTTON, LED_BLUE], schedule = [poll_gpio])]
    // fn poll_gpio() {
    //     // poll relevant GPIOs
    //     resources.USER_BUTTON.poll()
    //         .and_then(|event| {
    //             match event {
    //                 ButtonEvent::Up => {
    //                     info!("Button up!");
    //                     resources.LED_BLUE.off()
    //                 },
    //                 ButtonEvent::Down => resources.LED_BLUE.on(),
    //                 _ => Ok(())
    //             }
    //         })
    //         .unwrap();

    //     //schedule.poll_gpio(scheduled + GPIO_POLL_INTERVAL.cycles()).unwrap();
    // }

    // extern "C" {
    //     fn USART3();
    // }
};
