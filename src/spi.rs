use crate::error::Error;
use crate::{FtInner, PinUse};
use ftdi_mpsse::{ClockData, ClockDataOut, MpsseCmdBuilder, MpsseCmdExecutor};
use std::sync::{Arc, Mutex, MutexGuard};

/// FTDI SPI polarity.
///
/// This is a helper type to support multiple embedded-hal versions simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Polarity {
    /// MPSSE command used to clock data in and out simultaneously.
    ///
    /// This is set by [`Spi::set_clock_polarity`].
    clk: ClockData,
    /// MPSSE command used to clock data out.
    ///
    /// This is set by [`Spi::set_clock_polarity`].
    clk_out: ClockDataOut,
}

impl From<eh0::spi::Polarity> for Polarity {
    fn from(cpol: eh0::spi::Polarity) -> Self {
        match cpol {
            eh0::spi::Polarity::IdleLow => Polarity {
                clk: ClockData::MsbPosIn,
                clk_out: ClockDataOut::MsbNeg,
            },
            eh0::spi::Polarity::IdleHigh => Polarity {
                clk: ClockData::MsbNegIn,
                clk_out: ClockDataOut::MsbPos,
            },
        }
    }
}

impl From<eh1::spi::Polarity> for Polarity {
    fn from(cpol: eh1::spi::Polarity) -> Self {
        match cpol {
            eh1::spi::Polarity::IdleLow => Polarity {
                clk: ClockData::MsbPosIn,
                clk_out: ClockDataOut::MsbNeg,
            },
            eh1::spi::Polarity::IdleHigh => Polarity {
                clk: ClockData::MsbNegIn,
                clk_out: ClockDataOut::MsbPos,
            },
        }
    }
}

impl Default for Polarity {
    fn default() -> Self {
        Self {
            clk: ClockData::MsbPosIn,
            clk_out: ClockDataOut::MsbNeg,
        }
    }
}

/// FTDI SPI bus.
///
/// In embedded-hal version 1 this represents an exclusive SPI bus.
///
/// This is created by calling [`FtHal::spi`].
///
/// [`FtHal::spi`]: crate::FtHal::spi
#[derive(Debug)]
pub struct Spi<'a, Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: &'a Arc<Mutex<FtInner<Device>>>,
    /// SPI polarity
    pol: Polarity,
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
            pol: Default::default(),
        })
    }

    /// Set the SPI clock polarity.
    ///
    /// FTD2XX devices only supports [SPI mode] 0 and 2, clock phase is fixed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use eh1::spi::Polarity;
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
    pub fn set_clock_polarity<P: Into<Polarity>>(&mut self, cpol: P) {
        self.pol = cpol.into()
    }
}

impl<'a, Device, E> eh0::blocking::spi::Write<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn write(&mut self, words: &[u8]) -> Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.pol.clk_out, words)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.ft.send(cmd.as_slice())?;

        Ok(())
    }
}

impl<'a, Device, E> eh0::blocking::spi::Transfer<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.pol.clk, words)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.ft.send(cmd.as_slice())?;
        lock.ft.recv(words)?;

        Ok(words)
    }
}

impl<'a, Device, E> eh0::spi::FullDuplex<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn read(&mut self) -> nb::Result<u8, Error<E>> {
        let mut buf: [u8; 1] = [0];
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.pol.clk, &buf)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        match lock.ft.xfer(cmd.as_slice(), &mut buf) {
            Ok(()) => Ok(buf[0]),
            Err(e) => Err(nb::Error::Other(Error::from(e))),
        }
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.pol.clk_out, &[byte])
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        match lock.ft.send(cmd.as_slice()) {
            Ok(()) => Ok(()),
            Err(e) => Err(nb::Error::Other(Error::from(e))),
        }
    }
}

impl<E> eh1::spi::Error for Error<E>
where
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn kind(&self) -> eh1::spi::ErrorKind {
        eh1::spi::ErrorKind::Other
    }
}

impl<'a, Device, E> eh1::spi::ErrorType for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;
}

impl<'a, Device, E> eh1::spi::SpiBusFlush for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, Device, E> eh1::spi::SpiBusWrite<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn write(&mut self, words: &[u8]) -> Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.pol.clk_out, words)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.ft.send(cmd.as_slice())?;

        Ok(())
    }
}

impl<'a, Device, E> eh1::spi::SpiBusRead<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        let data_out: Vec<u8> = vec![0; words.len()];
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.pol.clk, &data_out)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.ft.send(cmd.as_slice())?;
        lock.ft.recv(words)?;

        Ok(())
    }
}

impl<'a, Device, E> eh1::spi::SpiBus<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.pol.clk, words)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        lock.ft.send(cmd.as_slice())?;
        lock.ft.recv(words)?;

        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.pol.clk, write)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.ft.send(cmd.as_slice())?;
        lock.ft.recv(read)?;

        Ok(())
    }
}

impl<'a, Device, E> ehnb1::spi::FullDuplex<u8> for Spi<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn read(&mut self) -> nb::Result<u8, Error<E>> {
        let mut buf: [u8; 1] = [0];
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.pol.clk, &buf)
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        match lock.ft.xfer(cmd.as_slice(), &mut buf) {
            Ok(()) => Ok(buf[0]),
            Err(e) => Err(nb::Error::Other(Error::from(e))),
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Error<E>> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.pol.clk_out, &[byte])
            .send_immediate();

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");
        match lock.ft.send(cmd.as_slice()) {
            Ok(()) => Ok(()),
            Err(e) => Err(nb::Error::Other(Error::from(e))),
        }
    }
}

pub struct SpiDeviceBus<'a, Device: MpsseCmdExecutor> {
    lock: MutexGuard<'a, FtInner<Device>>,
    pol: Polarity,
}

impl<'a, Device, E> eh1::spi::ErrorType for SpiDeviceBus<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;
}

impl<'a, Device, E> eh1::spi::SpiBusFlush for SpiDeviceBus<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, Device, E> eh1::spi::SpiBusRead<u8> for SpiDeviceBus<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.lock.ft.xfer(
            MpsseCmdBuilder::new()
                .clock_data(self.pol.clk, words)
                .send_immediate()
                .as_slice(),
            words,
        )?;
        Ok(())
    }
}

impl<'a, Device, E> eh1::spi::SpiBusWrite<u8> for SpiDeviceBus<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.lock.ft.send(
            MpsseCmdBuilder::new()
                .clock_data_out(self.pol.clk_out, words)
                .send_immediate()
                .as_slice(),
        )?;
        Ok(())
    }
}

impl<'a, Device, E> eh1::spi::SpiBus<u8> for SpiDeviceBus<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.lock.ft.xfer(
            MpsseCmdBuilder::new()
                .clock_data(self.pol.clk, write)
                .send_immediate()
                .as_slice(),
            read,
        )?;
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.lock.ft.xfer(
            MpsseCmdBuilder::new()
                .clock_data(self.pol.clk, words)
                .send_immediate()
                .as_slice(),
            words,
        )?;
        Ok(())
    }
}

/// FTDI SPI device, a SPI bus with chip select pin.
///
/// This is created by calling [`FtHal::spi_device`].
///
/// This is specific to embedded-hal version 1.
///
/// [`FtHal::spi_device`]: crate::FtHal::spi_device
#[derive(Debug)]
pub struct SpiDevice<'a, Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: &'a Arc<Mutex<FtInner<Device>>>,
    /// SPI polarity
    pol: Polarity,
    /// Chip select pin index.  0-7 for the FT232H.
    cs_idx: u8,
}

impl<'a, Device, E> SpiDevice<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(
        mtx: &'a Arc<Mutex<FtInner<Device>>>,
        cs_idx: u8,
    ) -> Result<SpiDevice<Device>, Error<E>> {
        let mut lock = mtx.lock().expect("Failed to aquire FTDI mutex");
        lock.allocate_pin(0, PinUse::Spi);
        lock.allocate_pin(1, PinUse::Spi);
        lock.allocate_pin(2, PinUse::Spi);
        lock.allocate_pin(cs_idx, PinUse::Output);

        let cs_mask: u8 = 1 << cs_idx;

        // clear direction of first 3 pins and CS
        lock.direction &= !(0x07 | cs_mask);
        // set SCK (AD0) and MOSI (AD1), and CS as output pins
        lock.direction |= 0x03 | cs_mask;

        // set GPIO pins to new state
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();
        lock.ft.send(cmd.as_slice())?;

        Ok(Self {
            mtx,
            pol: Default::default(),
            cs_idx,
        })
    }

    pub(crate) fn cs_mask(&self) -> u8 {
        1 << self.cs_idx
    }

    /// Set the SPI clock polarity.
    ///
    /// FTD2XX devices only supports [SPI mode] 0 and 2, clock phase is fixed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use eh1::spi::Polarity;
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// let mut spi = hal.spi_device(3)?;
    /// spi.set_clock_polarity(Polarity::IdleLow);
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [SPI mode]: https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
    pub fn set_clock_polarity<P: Into<Polarity>>(&mut self, cpol: P) {
        self.pol = cpol.into()
    }
}

impl<'a, Device, E> eh1::spi::ErrorType for SpiDevice<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;
}

impl<'a, Device, E> eh1::spi::SpiDevice for SpiDevice<'a, Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Bus = SpiDeviceBus<'a, Device>;

    fn transaction<R>(
        &mut self,
        f: impl FnOnce(&mut Self::Bus) -> Result<R, <Self::Bus as eh1::spi::ErrorType>::Error>,
    ) -> Result<R, Self::Error> {
        // lock the bus
        let mut lock: MutexGuard<FtInner<Device>> =
            self.mtx.lock().expect("Failed to aquire FTDI mutex");
        let direction: u8 = lock.direction;

        // assert the chip select pin
        let value_cs_asserted: u8 = lock.value & !self.cs_mask();
        lock.ft.send(
            MpsseCmdBuilder::new()
                .set_gpio_lower(value_cs_asserted, direction)
                .send_immediate()
                .as_slice(),
        )?;

        // call f with an exclusive reference to the bus
        let mut bus: SpiDeviceBus<Device> = SpiDeviceBus {
            lock,
            pol: self.pol,
        };
        let bus_result = f(&mut bus);

        // flush the bus
        {
            use eh1::spi::SpiBusFlush;
            bus.flush()?;
        }

        let mut lock: MutexGuard<FtInner<Device>> = bus.lock;

        // deassert the chip select pin
        let value_cs_deasserted: u8 = lock.value | self.cs_mask();
        lock.ft.send(
            MpsseCmdBuilder::new()
                .set_gpio_lower(value_cs_deasserted, direction)
                .send_immediate()
                .as_slice(),
        )?;

        // unlocking the bus is implicit via Drop
        bus_result
    }
}
