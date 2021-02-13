//! Inspired by [ftdi-embedded-hal] this is an [embedded-hal] implementation
//! for the for the FTDI chips using the [libftd2xx] drivers.
//!
//! This enables development of embedded devices drivers without the use of a
//! microcontroller.
//! The FTDI 2xx devices interface with your PC via USB.
//! They have a multi-protocol synchronous serial engine which allows them to
//! interface with most UART, SPI, and I2C embedded devices.
//!
//! # Setup
//!
//! One-time device setup instructions can be found in the [libftd2xx crate].
//!
//! # Examples
//!
//! * [newAM/eeprom25aa02e48-rs]
//!
//! # Limitations
//!
//! * Limited trait support: Blocking SPI and OutputPin traits are implemented.
//! * Limited device support: FT232H.
//! * No methods to control basic parameters such as clock frequency,
//!   USB latency, timeouts, ect...
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal
//! [ftdi-embedded-hal]: https://github.com/geomatsi/ftdi-embedded-hal
//! [libftd2xx crate]: https://github.com/newAM/libftd2xx-rs/
//! [libftd2xx]: https://github.com/newAM/libftd2xx-rs
//! [newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/master/examples/ftdi.rs
#![doc(html_root_url = "https://docs.rs/ftd2xx-embedded-hal/0.2.0")]
#![deny(unsafe_code, missing_docs)]

pub use embedded_hal;
pub use libftd2xx;

use embedded_hal::spi::Polarity;
use libftd2xx::{
    ClockData, ClockDataOut, DeviceTypeError, Ft232h, Ftdi, FtdiCommon, FtdiMpsse, MpsseCmdBuilder,
    MpsseSettings, TimeoutError,
};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::sync::Mutex;

/// FT232H device.
pub struct Ft232hHal {
    mtx: Mutex<RefCell<Ft232h>>,
    value: Mutex<u8>,
}

impl Ft232hHal {
    /// Create a new FT232H structure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftd2xx_embedded_hal as hal;
    ///
    /// let ftdi = hal::Ft232hHal::new()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Ft232hHal, DeviceTypeError> {
        let ft: Ft232h = Ft232h::try_from(&mut Ftdi::new()?)?;
        Ok(Self::with_ft(ft))
    }

    /// Create a new FT232H structure from a specific FT232H device.
    ///
    /// # Examples
    ///
    /// Selecting a device with a specific serial number.
    ///
    /// ```no_run
    /// use ftd2xx_embedded_hal as hal;
    /// use hal::libftd2xx::Ft232h;
    ///
    /// let ft = Ft232h::with_serial_number("FT59UO4C")?;
    /// let ftdi = hal::Ft232hHal::with_ft(ft);
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Selecting a device with a specific description.
    ///
    /// ```no_run
    /// use ftd2xx_embedded_hal as hal;
    /// use hal::libftd2xx::Ft232h;
    ///
    /// let ft = Ft232h::with_description("My device description")?;
    /// let ftdi = hal::Ft232hHal::with_ft(ft);
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_ft(ft: Ft232h) -> Ft232hHal {
        Ft232hHal {
            mtx: Mutex::new(RefCell::new(ft)),
            value: Mutex::new(0x00),
        }
    }

    /// Aquire the SPI peripheral for the FT232H.
    ///
    /// Pin assignments:
    /// * AD0 => SCK
    /// * AD1 => MOSI
    /// * AD2 => MISO
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftd2xx_embedded_hal as hal;
    ///
    /// let ftdi = hal::Ft232hHal::new()?;
    /// let spi = ftdi.spi()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn spi(&self) -> Result<Spi, TimeoutError> {
        Spi::new(&self.mtx)
    }

    /// Aquire the digital output pin 0 for the FT232H.
    pub fn ad0(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 0)
    }

    /// Aquire the digital output pin 1 for the FT232H.
    pub fn ad1(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 1)
    }

    /// Aquire the digital output pin 2 for the FT232H.
    pub fn ad2(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 2)
    }

    /// Aquire the digital output pin 3 for the FT232H.
    pub fn ad3(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 3)
    }

    /// Aquire the digital output pin 4 for the FT232H.
    pub fn ad4(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 4)
    }

    /// Aquire the digital output pin 5 for the FT232H.
    pub fn ad5(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 5)
    }

    /// Aquire the digital output pin 6 for the FT232H.
    pub fn ad6(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 6)
    }

    /// Aquire the digital output pin 7 for the FT232H.
    pub fn ad7(&self) -> OutputPin {
        OutputPin::new(&self.mtx, &self.value, 7)
    }
}

/// Output pin interface for FTD2xx devices.
pub struct OutputPin<'a> {
    mtx: &'a Mutex<RefCell<Ft232h>>,
    value: &'a Mutex<u8>,
    mask: u8,
}

/// SPI interface for FTD2xx devices.
pub struct Spi<'a> {
    mtx: &'a Mutex<RefCell<Ft232h>>,
    clk: ClockData,
    clk_out: ClockDataOut,
}

impl<'a> Spi<'a> {
    pub(crate) fn new(mtx: &Mutex<RefCell<Ft232h>>) -> Result<Spi, TimeoutError> {
        let lock = mtx.lock().unwrap();
        let mut ft = lock.borrow_mut();
        let mut settings = MpsseSettings::default();
        settings.mask = 0x1B;
        ft.initialize_mpsse(&settings)?;
        ft.set_clock(100_000)?;
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(0x18, 0x1B)
            .send_immediate();
        ft.write_all(cmd.as_slice())?;

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
    /// use ftd2xx_embedded_hal as hal;
    ///
    /// let ftdi = hal::Ft232hHal::new()?;
    /// let mut spi = ftdi.spi()?;
    /// spi.set_clock_polarity(Polarity::IdleLow);
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [SPI mode]: https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
    pub fn set_clock_polarity(&mut self, cpol: embedded_hal::spi::Polarity) {
        let (clk, clk_out) = match cpol {
            Polarity::IdleLow => (ClockData::MsbPosIn, ClockDataOut::MsbNeg),
            Polarity::IdleHigh => (ClockData::MsbNegIn, ClockDataOut::MsbPos),
        };

        // destructuring assignments are unstable
        self.clk = clk;
        self.clk_out = clk_out
    }
}

impl<'a> embedded_hal::blocking::spi::Write<u8> for Spi<'a> {
    type Error = TimeoutError;
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.clk_out, words)
            .send_immediate();

        let lock = self.mtx.lock().unwrap();
        let mut ft = lock.borrow_mut();
        ft.write_all(cmd.as_slice())
    }
}

impl<'a> embedded_hal::blocking::spi::Transfer<u8> for Spi<'a> {
    type Error = TimeoutError;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.clk, words)
            .send_immediate();

        let lock = self.mtx.lock().unwrap();
        let mut ft = lock.borrow_mut();
        ft.write_all(cmd.as_slice())?;
        ft.read_all(words)?;

        Ok(words)
    }
}

impl<'a> embedded_hal::spi::FullDuplex<u8> for Spi<'a> {
    type Error = TimeoutError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut buf: [u8; 1] = [0];
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data(self.clk, &buf)
            .send_immediate();

        let lock = self.mtx.lock().unwrap();
        let mut ft = lock.borrow_mut();
        ft.write_all(cmd.as_slice())?;
        ft.read_all(&mut buf)?;

        Ok(buf[0])
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Self::Error> {
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .clock_data_out(self.clk_out, &[byte])
            .send_immediate();

        let lock = self.mtx.lock().unwrap();
        let mut ft = lock.borrow_mut();
        ft.write_all(cmd.as_slice())?;
        Ok(())
    }
}

impl<'a> OutputPin<'a> {
    pub(crate) fn new(
        mtx: &'a Mutex<RefCell<Ft232h>>,
        value: &'a Mutex<u8>,
        bit: u8,
    ) -> OutputPin<'a> {
        let mask: u8 = 1 << bit;
        OutputPin { mtx, value, mask }
    }

    pub(crate) fn set(&self, state: bool) -> Result<(), TimeoutError> {
        let mut value = self.value.lock().unwrap();

        if state {
            *value |= self.mask;
        } else {
            *value &= !self.mask;
        };

        let lock = self.mtx.lock().unwrap();
        let mut ft = lock.borrow_mut();
        let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
            .set_gpio_lower(*value, 0x1B)
            .send_immediate();
        ft.write_all(cmd.as_slice())
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
