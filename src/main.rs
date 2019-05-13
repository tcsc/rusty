#![no_std]
#![no_main]

// RTFM macros need this for clean clippy report
#![allow(clippy::toplevel_ref_arg)]

extern crate panic_semihosting;
use stm32f407_audio::{self as audio, cs43l22};
use serial_log::SerialLogger;

mod types;
use types::*;

use rtfm::{app, Instant};
use cortex_m_semihosting::{hprintln}; // debug,
use heapless::consts::U256;

use stm32f4xx_hal::{
    prelude::*,
    stm32::{self, RCC, USART1},
    gpio::{self, Input, Output, PushPull, Floating,
        gpiob::{PB6, PB9},
        gpiod::{PD4, PD15},
    },
    i2c::I2c,
    rcc::Clocks,
    serial::{
        PinTx, PinRx, Serial,
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

static mut LOGGER : Option<SerialLogger<Usart1, U256>> = None;

fn init_logger<TX, RX>(uart: USART1, tx_pin: TX, rx_pin: RX, clocks: Clocks)
    -> SerialLogger<Serial<USART1, (TX,RX)>, U256>
        where TX : PinTx<USART1> + Send,
              RX : PinRx<USART1> + Send
{
    let config = SerialConfig {
        baudrate: 115_200.bps(),
        wordlength: WordLength::DataBits8,
        parity: Parity::ParityNone,
        stopbits: StopBits::STOP1
    };
    let serial_port = Serial::usart1(uart, (tx_pin, rx_pin), config, clocks).unwrap();
    SerialLogger::new(serial_port, log::Level::Info)
}

#[app(device = stm32f4xx_hal::stm32)]
const APP: () = {
    static mut USER_BUTTON : Button<UserButtonPin> = ();
    static mut LED_BLUE : Led<PD15<Output<PushPull>>> = ();
    static mut AUDIO_DEVICE : Cs43l22 = ();

    #[init(schedule = [poll_gpio])]
    fn init() {
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.mhz())
                             .sysclk(168.mhz())
                             .freeze();

        // initialise logging over UART1
        let gpio_port_a = device.GPIOA.split();
        let logger = init_logger(device.USART1,
                                 gpio_port_a.pa9.into_alternate_af7(),
                                 gpio_port_a.pa10.into_alternate_af7(),
                                 clocks);
        unsafe {
            crate::LOGGER = Some(logger);
            log::set_logger(LOGGER.as_ref().unwrap()).unwrap();
        }

        hprintln!("Setting I2S clock").unwrap();
        let rcc_registers = unsafe { &*RCC::ptr() };

        // prepare I2S PLL clock for 44.1 kHz output
        audio::set_i2s_clock(rcc_registers, 290, 2);
        hprintln!("I2S clock set").unwrap();

        // enable GPIOx port clocks
        rcc_registers.ahb1enr.write(|w| {
            w.gpioaen().bit(true) // user button
             .gpioben().bit(true) // I2C bus
             .gpiocen().bit(true)
             .gpioden().bit(true) // LEDs
        });

        // enable I2S clocks
        rcc_registers.apb1enr.write(|w| {
            w.i2c1en().bit(true)
             //.spi3en().bit(true)
             //.dma1en().bit(true)
        });

        // configure GPIO pin for user button
        let user_button_pin = gpio_port_a.pa0
                                         .into_pull_down_input();

        // schedule the GPIO polling task
        schedule.poll_gpio(Instant::now()).unwrap();

        let gpio_port_b = device.GPIOB.split();
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

    #[task(resources = [USER_BUTTON, LED_BLUE], schedule = [poll_gpio])]
    fn poll_gpio() {
        // poll relevant GPIOs
        resources.USER_BUTTON.poll()
            .and_then(|event| {
                match event {
                    ButtonEvent::Up => resources.LED_BLUE.off(),
                    ButtonEvent::Down => resources.LED_BLUE.on(),
                    _ => Ok(())
                }
            })
            .unwrap();

        schedule.poll_gpio(scheduled + GPIO_POLL_INTERVAL.cycles()).unwrap();
    }

    extern "C" {
        fn USART3();
    }
};