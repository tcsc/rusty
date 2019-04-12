#![no_std]
#![no_main]

// RTFM macros need this for clean clippy report
#![allow(clippy::toplevel_ref_arg)]

extern crate panic_semihosting;
use stm32f407_audio as audio;
use stm32f407g_disc as board;

use crate::board::{
    stm32::{RCC},
};

use rtfm::{app, Instant};
use cortex_m_semihosting::{hprintln}; // debug, 
use stm32f4xx_hal::{
    prelude::*, 
    gpio::{self, Input, Output, PullDown, PushPull, gpioa, gpiod}};

use portable::{Button, ButtonEvent, Led};

type UserButtonPin = gpioa::PA0<Input<PullDown>>;

const GPIO_POLL_INTERVAL : u32 = 840_000;

#[app(device = stm32f407g_disc)]
const APP: () = {
    static mut USER_BUTTON : Button<UserButtonPin> = ();
    static mut LED_BLUE : Led<gpiod::PD15<Output<PushPull>>> = ();

    #[init(schedule = [poll_gpio])]
    fn init() {
        hprintln!("Hello, world!").unwrap();
        
        hprintln!("Constraining RCC").unwrap();
        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.use_hse(8.mhz())
                              .sysclk(168.mhz())
                              .freeze();

        hprintln!("Setting I2S clock").unwrap();
        let rcc_registers = unsafe { &*RCC::ptr() };
    
        // prepare I2S PLL clock for 44.1 kHz output
        audio::set_i2s_clock(rcc_registers, 290, 2);
        hprintln!("I2S clock set").unwrap();

        // enable user button GPIOA & GPIOD port clocks
        rcc_registers.ahb1enr.write(|w| {
            w.gpioaen().bit(true);
            w.gpioden().bit(true)
        });

        // configure GPIO for user button
        let gpio_port_a = device.GPIOA.split();
        let user_button_pin = gpio_port_a.pa0
                                         .into_pull_down_input();

        // configure LED GPIO
        let gpio_port_d = device.GPIOD.split();
        let audio_reset_pin = gpio_port_d.pd4
                                         .into_push_pull_output()
                                         .set_speed(gpio::Speed::High);

        // schedule the GPIO polling task
        schedule.poll_gpio(Instant::now()).unwrap();
        
        // Add all of the resources 
        USER_BUTTON = Button::new(user_button_pin);
        LED_BLUE = Led::new(gpio_port_d.pd15
                                       .into_push_pull_output());
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