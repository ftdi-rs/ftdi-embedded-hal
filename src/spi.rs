use crate::error::Error;
use crate::{FtInner, PinUse};
use embedded_hal::spi::Polarity;
use ftdi_mpsse::{ClockData, ClockDataOut, MpsseCmdBuilder, MpsseCmdExecutor};
use std::sync::{Arc, Mutex};

/// FTDI SPI interface.
///
/// This is created by calling [`FtHal::spi`].
///
/// [`FtHal::spi`]: crate::FtHal::spi
#[derive(Debug)]
pub struct Spi<'a, Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: &'a Arc<Mutex<FtInner<Device>>>,
    /// MPSSE command used to clock data in and out simultaneously.
    ///
    /// This is set by [`Spi::set_clock_polarity`].
    clk: ClockData,
    /// MPSSE command used to clock data out.
    ///
    /// This is set by [`Spi::set_clock_polarity`].
    clk_out: ClockDataOut,
}

impl<'a, Device, E> Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(mtx: &Arc<Mutex<FtInner<Device>>>) -> Result<Spi<Device>, Error<E>> {
        let mut lock = mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.allocate_pin(0, PinUse::Spi);
        lock.allocate_pin(1, PinUse::Spi);
        lock.allocate_pin(2, PinUse::Spi);

        // clear direction of first 3 pins
        lock.direction &= !0x07;
        // set SCK (AD0) and MOSI (AD1) as output pins
        lock.direction |= 0x03;

        // set GPIO pins to new state
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();
        lock.ft.send(cmd.as_slice())?;

        Ok(Spi {
            mtx,
            clk: ClockData::MsbPosIn,
            clk_out: ClockDataOut::MsbNeg,
        })
    }

    /// Set the SPI clock polarity.
    ///
    /// FTD2XX devices only supports [SPI mode] 0 and 2, clock phase is fixed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use embedded_hal::spi::Polarity;
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// let mut spi = hal.spi()?;
    /// spi.set_clock_polarity(Polarity::IdleLow);
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [SPI mode]: https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
    pub fn set_clock_polarity(&mut self, cpol: Polarity) {
        let (clk, clk_out) = match cpol {
            Polarity::IdleLow => (ClockData::MsbPosIn, ClockDataOut::MsbNeg),
            Polarity::IdleHigh => (ClockData::MsbNegIn, ClockDataOut::MsbPos),
        };

        // destructuring assignments are unstable
        self.clk = clk;
        self.clk_out = clk_out
    }
}

impl<'a, Device, E> embedded_hal::blocking::spi::Write<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn write(&mut self, words: &[u8]) -> Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.clk_out, words)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        lock.ft.send(cmd.as_slice())?;

        Ok(())
    }
}

impl<'a, Device, E> embedded_hal::blocking::spi::Transfer<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.clk, words)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        lock.ft.send(cmd.as_slice())?;
        lock.ft.recv(words)?;

        Ok(words)
    }
}

impl<'a, Device, E> embedded_hal::spi::FullDuplex<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn read(&mut self) -> nb::Result<u8, Error<E>> {
        let mut buf: [u8; 1] = [0];
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.clk, &buf)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        match lock.ft.xfer(cmd.as_slice(), &mut buf) {
            Ok(()) => Ok(buf[0]),
            Err(e) => Err(nb::Error::Other(Error::from(e))),
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.clk_out, &[byte])
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        match lock.ft.send(cmd.as_slice()) {
            Ok(()) => Ok(()),
            Err(e) => Err(nb::Error::Other(Error::from(e))),
        }
    }
}
