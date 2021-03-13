use crate::{Ft232hInner, PinUse};
use libftd2xx::{FtdiCommon, MpsseCmdBuilder, TimeoutError};
use std::{cell::RefCell, sync::Mutex};

/// FTDI output pin.
#[derive(Debug)]
pub struct OutputPin<'a> {
    /// Parent FTDI device.
    mtx: &'a Mutex<RefCell<Ft232hInner>>,
    /// GPIO pin index.  0-7 for the FT232H.
    idx: u8,
}

impl<'a> OutputPin<'a> {
    pub(crate) fn new(mtx: &'a Mutex<RefCell<Ft232hInner>>, idx: u8) -> OutputPin<'a> {
        let lock = mtx.lock().expect("Failed to aquire FTDI mutex");
        let mut inner = lock.borrow_mut();
        inner.allocate_pin(idx, PinUse::Output);
        OutputPin { mtx, idx }
    }

    pub(crate) fn set(&self, state: bool) -> Result<(), TimeoutError> {
        let lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        let mut inner = lock.borrow_mut();

        if state {
            inner.value |= self.mask();
        } else {
            inner.value &= !self.mask();
        };

        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(inner.value, inner.direction)
            .send_immediate();
        inner.ft.write_all(cmd.as_slice())
    }

    /// Convert the GPIO pin index to a pin mask
    pub(crate) const fn mask(&self) -> u8 {
        1 << self.idx
    }
}

impl<'a> embedded_hal::digital::v2::OutputPin for OutputPin<'a> {
    type Error = TimeoutError;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set(false)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set(true)
    }
}
