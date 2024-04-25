use crate::error::Error;
use crate::{FtInner, PinUse};
use ftdi_mpsse::{MpsseCmdBuilder, MpsseCmdExecutor};
use std::sync::{Arc, Mutex};

/// Pin number
#[derive(Debug, Copy, Clone)]
pub(crate) enum Pin {
    Lower(u8),
    Upper(u8),
}

/// FTDI output pin.
///
/// This is created by calling [`FtHal::ad0`] - [`FtHal::ad7`].
///
/// [`FtHal::ad0`]: crate::FtHal::ad0
/// [`FtHal::ad7`]: crate::FtHal::ad7
#[derive(Debug)]
pub struct OutputPin<Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: Arc<Mutex<FtInner<Device>>>,
    /// GPIO pin index.  0-7 for the FT232H.
    pin: Pin,
}

impl<Device, E> OutputPin<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(
        mtx: Arc<Mutex<FtInner<Device>>>,
        pin: Pin,
    ) -> Result<OutputPin<Device>, Error<E>> {
        {
            let mut lock = mtx.lock().expect("Failed to aquire FTDI mutex");

            lock.allocate_pin_any(pin, PinUse::Output);

            let (byte, idx) = match pin {
                Pin::Lower(idx) => (&mut lock.lower, idx),
                Pin::Upper(idx) => (&mut lock.upper, idx),
            };
            byte.direction |= 1 << idx;
            let cmd = MpsseCmdBuilder::new();
            let cmd = match pin {
                Pin::Lower(_) => cmd.set_gpio_lower(byte.value, byte.direction),
                Pin::Upper(_) => cmd.set_gpio_upper(byte.value, byte.direction),
            }
            .send_immediate();
            lock.ft.send(cmd.as_slice())?;
        }
        Ok(OutputPin { mtx, pin })
    }

    pub(crate) fn set(&self, state: bool) -> Result<(), Error<E>> {
        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        let byte = match self.pin {
            Pin::Lower(_) => &mut lock.lower,
            Pin::Upper(_) => &mut lock.upper,
        };

        if state {
            byte.value |= self.mask();
        } else {
            byte.value &= !self.mask();
        };

        let cmd = MpsseCmdBuilder::new();
        let cmd = match self.pin {
            Pin::Lower(_) => cmd.set_gpio_lower(byte.value, byte.direction),
            Pin::Upper(_) => cmd.set_gpio_upper(byte.value, byte.direction),
        }
        .send_immediate();
        lock.ft.send(cmd.as_slice())?;

        Ok(())
    }
}

impl<Device: MpsseCmdExecutor> OutputPin<Device> {
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        let idx = match self.pin {
            Pin::Lower(idx) => idx,
            Pin::Upper(idx) => idx,
        };
        1 << idx
    }
}

impl<Device, E> eh1::digital::ErrorType for OutputPin<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;
}

impl<Device, E> eh1::digital::OutputPin for OutputPin<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn set_low(&mut self) -> Result<(), Error<E>> {
        self.set(false)
    }

    fn set_high(&mut self) -> Result<(), Error<E>> {
        self.set(true)
    }
}

impl<Device, E> eh0::digital::v2::OutputPin for OutputPin<Device>
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
pub struct InputPin<Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: Arc<Mutex<FtInner<Device>>>,
    /// GPIO pin index.  0-7 for the FT232H.
    pin: Pin,
}

impl<Device, E> InputPin<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(
        mtx: Arc<Mutex<FtInner<Device>>>,
        pin: Pin,
    ) -> Result<InputPin<Device>, Error<E>> {
        {
            let mut lock = mtx.lock().expect("Failed to aquire FTDI mutex");

            lock.allocate_pin_any(pin, PinUse::Input);

            let (byte, idx) = match pin {
                Pin::Lower(idx) => (&mut lock.lower, idx),
                Pin::Upper(idx) => (&mut lock.upper, idx),
            };
            byte.direction &= !(1 << idx);
            let cmd = MpsseCmdBuilder::new();
            let cmd = match pin {
                Pin::Lower(_) => cmd.set_gpio_lower(byte.value, byte.direction),
                Pin::Upper(_) => cmd.set_gpio_upper(byte.value, byte.direction),
            }
            .send_immediate();
            lock.ft.send(cmd.as_slice())?;
        }
        Ok(InputPin { mtx, pin })
    }

    pub(crate) fn get(&self) -> Result<bool, Error<E>> {
        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        let mut buffer = [0u8; 1];
        let cmd = MpsseCmdBuilder::new();
        let cmd = match self.pin {
            Pin::Lower(_) => cmd.gpio_lower(),
            Pin::Upper(_) => cmd.gpio_upper(),
        }
        .send_immediate();
        lock.ft.send(cmd.as_slice())?;
        lock.ft.recv(&mut buffer)?;

        Ok((buffer[0] & self.mask()) != 0)
    }
}

impl<Device: MpsseCmdExecutor> InputPin<Device> {
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        let idx = match self.pin {
            Pin::Lower(idx) => idx,
            Pin::Upper(idx) => idx,
        };
        1 << idx
    }
}

impl<Device, E> eh1::digital::ErrorType for InputPin<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;
}

impl<Device, E> eh1::digital::InputPin for InputPin<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.get()
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.get().map(|res| !res)
    }
}

impl<Device, E> eh0::digital::v2::InputPin for InputPin<Device>
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
