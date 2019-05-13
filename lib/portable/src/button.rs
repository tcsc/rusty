
use embedded_hal::{
    digital::v2::InputPin
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    None,
    Up,
    Down
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum SignalState {
    Low,
    High,
}

pub struct Button<InputType> {
    pin: InputType,
    state: SignalState,
    contra_count: u8
}

const SWITCH_THRESHOLD : u8 = 5;

impl<T> Button<T> where T : InputPin {

    pub fn new(pin: T) -> Button<T> {
        Button {pin, state: SignalState::High, contra_count: 0}
    }

    pub fn poll(&mut self) -> Result<Event, T::Error> {
        use SignalState::*;

        let pin_is_high = self.pin.is_high()?;
        let pin_is_low = !pin_is_high;

        // hprintln!("Button state: {:?}, signal: {}, contra_count: {}",
        //     self.state,
        //     if pin_is_high { "high" } else { "low" },
        //     self.contra_count ).unwrap();

        match self.state {
            High if pin_is_high => self.contra_count = 0,
            High if pin_is_low => self.contra_count += 1,
            Low if pin_is_low => self.contra_count = 0,
            Low if pin_is_high => self.contra_count += 1,
            _ => {
                panic!("This should bever happen state")
            }
        }

        let event = match self.state {
            High if self.contra_count >= SWITCH_THRESHOLD => {
                self.state = Low;
                self.contra_count = 0;
                Event::Up
            },

            Low if self.contra_count >= SWITCH_THRESHOLD => {
                self.state = High;
                self.contra_count = 0;
                Event::Down
            }

            _ => Event::None
        };

        Ok(event)
    }

    pub fn is_up(&self) -> bool {
        self.state == SignalState::Low
    }

    pub fn is_down(&self) -> bool {
        self.state == SignalState::High
    }
}