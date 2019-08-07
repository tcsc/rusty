#![no_std]


use bare_metal::Mutex;
use cortex_m::interrupt;
use nb;
use embedded_hal as hal;
use heapless::{ArrayLength};
use core::{
    cell::RefCell,
    fmt::Write,
    marker::PhantomData
};

pub struct SerialLogger<SerialT, N>
{
    loglevel: log::Level,
    serial_port: Mutex<RefCell<SerialT>>,
    _n: PhantomData<N>
}

impl<SerialT, N> SerialLogger<SerialT, N>
    where SerialT : hal::serial::Write<u8> + Send,
          N : ArrayLength<u8>
{
    pub fn new(serial_port: SerialT, loglevel: log::Level)
        -> SerialLogger<SerialT, N>
    {
        SerialLogger {
            loglevel,
            serial_port: Mutex::new(RefCell::new(serial_port)),
            _n: PhantomData
        }
    }

    // locks the serial port
    fn with_serial_port<T>(&self, f: impl FnOnce(&mut SerialT) -> T) -> T {
        interrupt::free(|cs| {
            let cell = self.serial_port.borrow(cs);
            let mut serial_port = cell.borrow_mut();
            f(&mut serial_port)
        })
    }
}

fn writeln<S>(s: &mut S, buffer: &[u8]) -> Result<(), S::Error>
    where S : hal::serial::Write<u8>
{
    for &b in buffer {
        nb::block!(s.write(b))?
    }
    nb::block!(s.write(0x0D))?;
    nb::block!(s.write(0x0A))?;
    Ok(())
}

impl<SerialT, N> log::Log for SerialLogger<SerialT, N>
    where SerialT : hal::serial::Write<u8> + Send,
          N : ArrayLength<u8> + Sync + Send
{
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.loglevel
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut buffer : heapless::String<N> = heapless::String::new();
            if buffer.write_fmt(record.args().clone()).is_ok() {
                self.with_serial_port(|serial_port|{
                    let _ = writeln(serial_port, buffer.as_bytes());
                })
            }
        }
    }

    fn flush(&self) {
        self.with_serial_port(|serial_port| {
            let _ = serial_port.flush();
        })
    }
}