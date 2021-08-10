use crate::{FtInner, PinUse};
use libftd2xx::{FtdiCommon, MpsseCmdBuilder, TimeoutError};
use std::{cell::RefCell, sync::Mutex};

/// FTDI output pin.
///
/// This is created by calling [`FtHal::ad0`] - [`FtHal::ad7`].
///
/// [`FtHal::ad0`]: crate::FtHal::ad0
/// [`FtHal::ad7`]: crate::FtHal::ad7
#[derive(Debug)]
pub struct OutputPin<'a, Device: FtdiCommon> {
    /// Parent FTDI device.
    mtx: &'a Mutex<RefCell<FtInner<Device>>>,
    /// GPIO pin index.  0-7 for the FT232H.
    idx: u8,
}

impl<'a, Device: FtdiCommon> OutputPin<'a, Device> {
    pub(crate) fn new(mtx: &'a Mutex<RefCell<FtInner<Device>>>, idx: u8) -> OutputPin<'a, Device> {
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
}

impl<'a, Device: FtdiCommon> OutputPin<'a, Device> {
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        1 << self.idx
    }
}

impl<'a, Device: FtdiCommon> embedded_hal::digital::v2::OutputPin for OutputPin<'a, Device> {
    type Error = TimeoutError;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set(false)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set(true)
    }
}
