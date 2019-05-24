use log::{info};

use stm32f4xx_hal::{
    hal::{
        digital::OutputPin,
        blocking::i2c::{Write, WriteRead}
    },
};

#[derive(Clone, Copy, Debug)]
pub enum Register {
    ChipId = 0x01,
}

impl Register {
    fn addr(self) -> u8 { self as u8 }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Revision {
    A0 = 0b000,
    A1 = 0b001,
    B0 = 0b010,
    B1 = 0b011
}

impl Revision {
    fn from_u8(val: u8) -> Option<Revision> {
        match val {
            0b000 => Some(Revision::A0),
            0b001 => Some(Revision::A1),
            0b010 => Some(Revision::B0),
            0b011 => Some(Revision::B1),
            _ => None
        }
    }
}

pub struct Driver<I2CT, ResetPinT> {
    i2c: I2CT,
    addr: u8,
    reset_pin: ResetPinT
}

#[derive(Debug)]
pub enum DriverError {
    Comm,
    ChipId,
    Value,
}

impl<I2CT, ResetPinT, E> Driver<I2CT, ResetPinT>
    where ResetPinT : OutputPin,
          I2CT :  WriteRead<Error = E> + Write<Error = E>,
{
    pub fn init(i2c: I2CT, addr: u8, reset_pin: ResetPinT)
            -> Result<Driver<I2CT, ResetPinT>, DriverError> {
        let mut d = Driver { i2c, addr, reset_pin };
        d.power_on();
        let id = d.chip_id()?;
        info!("CS43L22 Revision: {:?}", id);
        Ok(d)
    }

    pub fn power_on(&mut self) {
        self.reset_pin.set_high();
    }

    pub fn power_off(&mut self) {
        self.reset_pin.set_low();
    }

    fn chip_id(&mut self) -> Result<Revision, DriverError> {
        let mut r : [u8; 1] = [0];

        info!("Reading register...");
        self.i2c.write_read(self.addr, &[Register::ChipId.addr()], &mut r)
                .map_err(|_| DriverError::Comm)?;
        let register = r[0];

        if (register & 0xF8) != 0xE0 {
            return Err(DriverError::ChipId);
        }

        Revision::from_u8(register & 0x07).ok_or(DriverError::Value)
    }
}
