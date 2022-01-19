use crate::error::Error;
use crate::{FtInner, PinUse};
use ftdi_mpsse::{MpsseCmdBuilder, MpsseCmdExecutor};
use std::result::Result;
use std::{cell::RefCell, sync::Mutex};

/// FTDI output pin.
///
/// This is created by calling [`FtHal::ad0`] - [`FtHal::ad7`].
///
/// [`FtHal::ad0`]: crate::FtHal::ad0
/// [`FtHal::ad7`]: crate::FtHal::ad7
#[derive(Debug)]
pub struct OutputPin<'a, Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: &'a Mutex<RefCell<FtInner<Device>>>,
    /// GPIO pin index.  0-7 for the FT232H.
    idx: u8,
}

impl<'a, Device, E> OutputPin<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(
        mtx: &'a Mutex<RefCell<FtInner<Device>>>,
        idx: u8,
    ) -> Result<OutputPin<'a, Device>, Error<E>> {
        let lock = mtx.lock().expect("Failed to aquire FTDI mutex");
        let mut inner = lock.borrow_mut();
        inner.direction |= 1 << idx;
        inner.allocate_pin(idx, PinUse::Output);
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(inner.value, inner.direction)
            .send_immediate();
        inner.ft.send(cmd.as_slice())?;
        Ok(OutputPin { mtx, idx })
    }

    pub(crate) fn set(&self, state: bool) -> Result<(), Error<E>> {
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
        inner.ft.send(cmd.as_slice())?;

        Ok(())
    }
}

impl<'a, Device: MpsseCmdExecutor> OutputPin<'a, Device> {
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        1 << self.idx
    }
}

impl<'a, Device, E> embedded_hal::digital::v2::OutputPin for OutputPin<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn set_low(&mut self) -> Result<(), Error<E>> {
        self.set(false)
    }

    fn set_high(&mut self) -> Result<(), Error<E>> {
        self.set(true)
    }
}

/// FTDI input pin.
///
/// This is created by calling [`FtHal::adi0`] - [`FtHal::adi7`].
///
/// [`FtHal::adi0`]: crate::FtHal::adi0
/// [`FtHal::adi7`]: crate::FtHal::adi7
#[derive(Debug)]
pub struct InputPin<'a, Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: &'a Mutex<RefCell<FtInner<Device>>>,
    /// GPIO pin index.  0-7 for the FT232H.
    idx: u8,
}

impl<'a, Device, E> InputPin<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(
        mtx: &'a Mutex<RefCell<FtInner<Device>>>,
        idx: u8,
    ) -> Result<InputPin<'a, Device>, Error<E>> {
        let lock = mtx.lock().expect("Failed to aquire FTDI mutex");
        let mut inner = lock.borrow_mut();
        inner.direction &= !(1 << idx);
        inner.allocate_pin(idx, PinUse::Input);
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(inner.value, inner.direction)
            .send_immediate();
        inner.ft.send(cmd.as_slice())?;
        Ok(InputPin { mtx, idx })
    }

    pub(crate) fn get(&self) -> Result<bool, Error<E>> {
        let lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        let mut inner = lock.borrow_mut();

        let mut buffer = [0u8; 1];
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new().gpio_lower().send_immediate();
        inner.ft.send(cmd.as_slice())?;
        inner.ft.recv(&mut buffer)?;

        Ok((buffer[0] & self.mask()) != 0)
    }
}

impl<'a, Device: MpsseCmdExecutor> InputPin<'a, Device> {
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        1 << self.idx
    }
}

impl<'a, Device, E> embedded_hal::digital::v2::InputPin for InputPin<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.get()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.get().map(|res| !res)
    }
}
