#![no_std]
#![no_main]

// RTFM macros need this for clean clippy report
#![allow(clippy::toplevel_ref_arg)]

mod button;

extern crate panic_semihosting;
use stm32f407_audio as audio;
use stm32f407g_disc as board;

use crate::board::{
    led::{LedColor, Leds},
    stm32::{RCC},
};

use rtfm::{app, Instant};
use cortex_m_semihosting::{hprintln}; // debug, 
use stm32f4xx_hal::{prelude::*, gpio};

use button::Button;

type UserButtonPin = gpio::gpioa::PA0<gpio::Input<gpio::PullDown>>;

const GPIO_POLL_INTERVAL : u32 = 840_000;

#[app(device = stm32f407g_disc)]
const APP: () = {
    static mut USER_BUTTON : button::Button<UserButtonPin> = ();
    static mut LEDS : Leds = ();

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

        // enable user button gpio clock
        rcc_registers.ahb1enr.write(|w| w.gpioaen().bit(true) );

        // configure GPIO for user button
        let gpio_port_a = device.GPIOA.split();
        let user_button_pin = gpio_port_a.pa0.into_pull_down_input();

        // configure LED bank
        let gpio_port_d = device.GPIOD.split();

        // start a timer for polling the GPIO
        // let mut button_timer = Timer::tim3(device.TIM3, 1.khz(), clocks);
        // button_timer.listen(timer::Event::TimeOut);

        schedule.poll_gpio(Instant::now()).unwrap();
        
        USER_BUTTON = Button::new(user_button_pin);
        LEDS = Leds::new(gpio_port_d);
    }

    #[task(resources = [USER_BUTTON, LEDS], schedule = [poll_gpio])]
    fn poll_gpio() {
        // poll relevant GPIOs
        match resources.USER_BUTTON.poll() {
            button::Event::Up => {
                resources.LEDS[LedColor::Blue].off()
            },
            button::Event::Down => {
                resources.LEDS[LedColor::Blue].on()
            },
            _ => ()
        }

        schedule.poll_gpio(scheduled + GPIO_POLL_INTERVAL.cycles()).unwrap();
    }

    // #[interrupt(resources = [USER_BUTTON, LEDS])]
    // fn TIM3() {
    //     // clear timer interrupt
    //     let timer = unsafe { &*stm32::TIM3::ptr() };
    //     timer.sr.modify(|_, w| w.uif().clear_bit());

     
    // }

    extern "C" {
        fn USART1();
    }
};