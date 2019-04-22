#![no_std]
#![no_main]

// RTFM macros need this for clean clippy report
#![allow(clippy::toplevel_ref_arg)]

extern crate panic_semihosting;
use stm32f407_audio::{self as audio, cs43l22};

use rtfm::{app, Instant};
use cortex_m_semihosting::{hprintln}; // debug,
use stm32f4xx_hal::{
    prelude::*,
    stm32::{self, RCC},
    gpio::{self, Input, Output, PullDown, PushPull, Floating, gpioa, gpiob, gpiod, AF4},
    i2c::{I2c, Pins}
};

use portable::{Button, ButtonEvent, Led};

type UserButtonPin = gpioa::PA0<Input<PullDown>>;
type I2C1SclPin =  gpiob::PB6<gpio::Alternate<AF4>>;
type I2C1SdaPin = gpiob::PB9<gpio::Alternate<AF4>>;
type I2CBus1 = I2c<stm32::I2C1, (I2C1SclPin, I2C1SdaPin)>;
type AudioResetPin = gpiod::PD4<Output<PushPull>>;
type Cs43l22 = cs43l22::Driver<I2CBus1, AudioResetPin>;

const GPIO_POLL_INTERVAL : u32 = 840_000;
const AUDIO_DEVICE_ADDR : u8 = 0x4a;

fn init_audio(
    i2c1: stm32::I2C1,
    scl: gpiob::PB6<Input<Floating>>,
    sda: gpiob::PB9<Input<Floating>>,
    audio_reset: gpiod::PD4<Input<Floating>>,
    clocks: stm32f4xx_hal::rcc::Clocks)
        -> Cs43l22
{
    let scl = scl.into_alternate_af4()
                 .set_speed(gpio::Speed::High)
                 .set_open_drain();

    let sda = sda.into_alternate_af4()
                 .set_speed(gpio::Speed::High)
                 .set_open_drain();

    let audio_reset = audio_reset.into_push_pull_output()
                                 .set_speed(gpio::Speed::High);

    let audio_i2c = I2c::i2c1(i2c1, (scl, sda), 100.khz(), clocks);

    audio::cs43l22::Driver::init(audio_i2c,
        AUDIO_DEVICE_ADDR,
        audio_reset).unwrap()
}

#[app(device = stm32f407g_disc)]
const APP: () = {
    static mut USER_BUTTON : Button<UserButtonPin> = ();
    static mut LED_BLUE : Led<gpiod::PD15<Output<PushPull>>> = ();
    static mut AUDIO_DEVICE : Cs43l22 = ();

    #[init(schedule = [poll_gpio])]
    fn init() {
        hprintln!("Hello, world!").unwrap();

        hprintln!("Constraining RCC").unwrap();
        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(8.mhz())
                             .sysclk(168.mhz())
                             .freeze();

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

        // configure GPIO pin for user button
        let gpio_port_a = device.GPIOA.split();
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
        USER_BUTTON = Button::new(user_button_pin);
        LED_BLUE = Led::new(gpio_port_d.pd15
                                       .into_push_pull_output());
        AUDIO_DEVICE = audio_device;
    }

    #[task(resources = [USER_BUTTON, LED_BLUE], schedule = [poll_gpio])]
    fn poll_gpio() {
        // poll relevant GPIOs
        match resources.USER_BUTTON.poll() {
            ButtonEvent::Up => {
                resources.LED_BLUE.off()
            },
            ButtonEvent::Down => {
                resources.LED_BLUE.on()
            },
            _ => ()
        }

        schedule.poll_gpio(scheduled + GPIO_POLL_INTERVAL.cycles()).unwrap();
    }

    extern "C" {
        fn USART1();
    }
};