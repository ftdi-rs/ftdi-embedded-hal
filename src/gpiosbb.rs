use crate::error::Error;
use crate::fthalsbb::FtInnerSbb;
use crate::PinUse;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub use ftdi;

// ----------------------------------------------------------------------------
// Taken from ftdi-embedded-hal without change.

/// Pin number
#[derive(Debug, Copy, Clone)]
pub(crate) enum Pin {
    Lower(u8),
    Upper(u8),
}

// ----------------------------------------------------------------------------

/// FTDI output pin.
///
/// This is created by calling [`FtHalSbb::ad0`] - [`FtHalSbb::ad7`].
///
/// [`FtHalSbb::ad0`]: crate::FtHalSbb::ad0
/// [`FtHalSbb::ad7`]: crate::FtHalSbb::ad7
pub struct OutputPinSbb<E> {
    /// Parent FTDI device.
    mtx: Arc<Mutex<FtInnerSbb>>,
    /// GPIO pin index.  0-7 for the FTx232H.
    pin: Pin,
    // Satisfy the compiler.
    _p: PhantomData<E>,
}

impl<E> OutputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    pub(crate) fn new(mtx: Arc<Mutex<FtInnerSbb>>, pin: Pin) -> Result<OutputPinSbb<E>, Error<E>> {
        {
            let mut lock = mtx.lock().expect("Failed to acquire FTDI mutex");

            lock.allocate_pin_any(pin, PinUse::Output);

            let (byte, idx) = match pin {
                Pin::Lower(idx) => (&mut lock.lower, idx),
                Pin::Upper(idx) => (&mut lock.upper, idx),
            };
            byte.direction |= 1 << idx;

            let out_mask = byte.direction;

            match pin {
                Pin::Lower(_) => lock.ft.set_bitmode(out_mask, ftdi::BitMode::SyncBB)?,
                Pin::Upper(_) => panic!("Upper byte not supported by FtHalSbb."),
            }
        }
        Ok(OutputPinSbb {
            mtx,
            pin,
            _p: PhantomData,
        })
    }

    pub(crate) fn set(&self, state: bool) -> Result<(), Error<E>> {
        let mut lock = self.mtx.lock().expect("Failed to acquire FTDI mutex");

        let byte = match self.pin {
            Pin::Lower(_) => &mut lock.lower,
            Pin::Upper(_) => &mut lock.upper,
        };

        if state {
            byte.value |= self.mask();
        } else {
            byte.value &= !self.mask();
        };

        let out_buf = [byte.value];

        // Read the GPIO pin states (from the parallel I/O port itself) and
        // discard. This entire-buffer read makes sure that the USB TX FIFO in
        // the chip can't fill-up due to executing many writes without a read
        // in sequence. It also avoids any accumulation of stray bytes in the
        // buffer. (This has never happened after the initial purge thus far,
        // but it is a safeguard.)
        let mut read_bytes = vec![];
        lock.ft.read_to_end(&mut read_bytes)?;

        // Write the new pin states to the output.
        lock.ft.write_all(&out_buf)?;

        Ok(())
    }
}

impl<E> OutputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        let idx = match self.pin {
            Pin::Lower(idx) => idx,
            Pin::Upper(idx) => idx,
        };
        1 << idx
    }
}

impl<E> eh1::digital::ErrorType for OutputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    type Error = Error<E>;
}

impl<E> eh1::digital::OutputPin for OutputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    fn set_low(&mut self) -> Result<(), Error<E>> {
        self.set(false)
    }

    fn set_high(&mut self) -> Result<(), Error<E>> {
        self.set(true)
    }
}

/// FTDI input pin.
///
/// This is created by calling [`FtHalSbb::adi0`] - [`FtHalSbb::adi7`].
///
/// [`FtHalSbb::adi0`]: crate::FtHalSbb::adi0
/// [`FtHalSbb::adi7`]: crate::FtHalSbb::adi7
pub struct InputPinSbb<E> {
    /// Parent FTDI device.
    mtx: Arc<Mutex<FtInnerSbb>>,
    /// GPIO pin index.  0-7 for the FTx232H.
    pin: Pin,
    // Satisfy the compiler.
    _p: PhantomData<E>,
}

impl<E> InputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    pub(crate) fn new(mtx: Arc<Mutex<FtInnerSbb>>, pin: Pin) -> Result<InputPinSbb<E>, Error<E>> {
        {
            let mut lock = mtx.lock().expect("Failed to acquire FTDI mutex");

            lock.allocate_pin_any(pin, PinUse::Input);

            let (byte, idx) = match pin {
                Pin::Lower(idx) => (&mut lock.lower, idx),
                Pin::Upper(idx) => (&mut lock.upper, idx),
            };
            byte.direction &= !(1 << idx);

            let out_mask = byte.direction;

            match pin {
                Pin::Lower(_) => lock.ft.set_bitmode(out_mask, ftdi::BitMode::SyncBB)?,
                Pin::Upper(_) => panic!("Upper byte not supported by FtHalSbb."),
            }
        }
        Ok(InputPinSbb {
            mtx,
            pin,
            _p: PhantomData,
        })
    }

    pub(crate) fn get(&self) -> Result<bool, Error<E>> {
        let mut lock = self.mtx.lock().expect("Failed to acquire FTDI mutex");

        // The read can return empty if the chip's USB TX buffer is empty.
        // Because the bit-bang is synchronous, a write is required to obtain
        // data to read-back. Thus, two reads in a row need a write in-between.
        // Also, the last byte written should be doubled-up, so that the read
        // (which has a one byte delay) is synchronous again.
        //
        // Thus, by always writing-out the output state before a read, both
        // requirements satisfied.
        let byte = match self.pin {
            Pin::Lower(_) => &mut lock.lower,
            Pin::Upper(_) => &mut lock.upper,
        };

        let out_buf = [byte.value];
        lock.ft.write_all(&out_buf)?;

        // Read the GPIO pin states (from the parallel I/O port itself).
        // All bytes in the buffer are taken-in, discarding all but the last
        // one, which is the result of writing the previous bit pattern.
        // This entire-buffer read makes sure that the USB TX FIFO in the chip
        // can't fill-up due to cumulative weirdness. (This has never happened
        // after the initial purge thus far, but it is a safeguard.)
        let mut read_bytes = vec![];
        lock.ft.read_to_end(&mut read_bytes)?;
        let pin_states = *(read_bytes.last().unwrap());

        Ok((pin_states & self.mask()) != 0)
    }
}

impl<E> InputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    /// Convert the GPIO pin index to a pin mask
    pub(crate) fn mask(&self) -> u8 {
        let idx = match self.pin {
            Pin::Lower(idx) => idx,
            Pin::Upper(idx) => idx,
        };
        1 << idx
    }
}

impl<E> eh1::digital::ErrorType for InputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    type Error = Error<E>;
}

impl<E> eh1::digital::InputPin for InputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.get()
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.get().map(|res| !res)
    }
}

impl<E> eh0::digital::v2::InputPin for InputPinSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    type Error = Error<E>;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.get()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.get().map(|res| !res)
    }
}
