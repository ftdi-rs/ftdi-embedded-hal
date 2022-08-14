//! This is an [embedded-hal] implementation for the FTDI chips
//! that can use various drivers including [libftd2xx] and [ftdi-rs].
//!
//! This enables development of embedded device drivers without the use of
//! a microcontroller. The FTDI devices interface with PC via USB, and
//! provide a multi-protocol synchronous serial engine to interface
//! with most GPIO, SPI, I2C embedded devices.
//!
//! **Note:**
//! This is strictly a development tool.
//! The crate contains runtime borrow checks and explicit panics to adapt the
//! FTDI device into the [embedded-hal] traits.
//!
//! # Quickstart
//!
//! * Enable the "libftd2xx-static" feature flag to use static linking with libftd2xx driver.
//! * Linux users only: Add [udev rules].
//!
//! ```toml
//! [dependencies.ftdi-embedded-hal]
//! version = "0.11"
//! features = ["libftd2xx", "libftd2xx-static"]
//! ```
//!
//! # Limitations
//!
//! * Limited trait support: SPI, I2C, Delay, InputPin, and OutputPin traits are implemented.
//! * Limited device support: FT232H, FT2232H, FT4232H.
//! * Limited SPI modes support: MODE0, MODE2.
//!
//! # Examples
//!
//! ## SPI
//!
//! Communicate with SPI devices using [ftdi-rs] driver:
//! ```no_run
//! use ftdi_embedded_hal as hal;
//!
//! # #[cfg(feature = "ftdi")]
//! # {
//! let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
//!     .interface(ftdi::Interface::A)
//!     .open()?;
//!
//! let hal = hal::FtHal::init_freq(device, 3_000_000)?;
//! let spi = hal.spi()?;
//! # }
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! Communicate with SPI devices using [libftd2xx] driver:
//! ```no_run
//! use ftdi_embedded_hal as hal;
//!
//! # #[cfg(feature = "libftd2xx")]
//! # {
//! let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;
//!
//! let hal = hal::FtHal::init_freq(device, 3_000_000)?;
//! let spi = hal.spi()?;
//! # }
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! ## I2C
//!
//! Communicate with I2C devices using [ftdi-rs] driver:
//! ```no_run
//! use ftdi_embedded_hal as hal;
//!
//! # #[cfg(feature = "ftdi")]
//! # {
//! let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
//!     .interface(ftdi::Interface::A)
//!     .open()?;
//!
//! let hal = hal::FtHal::init_freq(device, 400_000)?;
//! let i2c = hal.i2c()?;
//! # }
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! Communicate with I2C devices using [libftd2xx] driver:
//! ```no_run
//! use ftdi_embedded_hal as hal;
//!
//! # #[cfg(feature = "libftd2xx")]
//! # {
//! let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
//!
//! let hal = hal::FtHal::init_freq(device, 400_000)?;
//! let i2c = hal.i2c()?;
//! # }
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! ## GPIO
//!
//! Control GPIO pins using [libftd2xx] driver:
//! ```no_run
//! use ftdi_embedded_hal as hal;
//!
//! # #[cfg(feature = "libftd2xx")]
//! # {
//! let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
//!
//! let hal = hal::FtHal::init_default(device)?;
//! let gpio = hal.ad6();
//! # }
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! Control GPIO pins using [ftdi-rs] driver:
//! ```no_run
//! use ftdi_embedded_hal as hal;
//!
//! # #[cfg(feature = "ftdi")]
//! # {
//! let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
//!     .interface(ftdi::Interface::A)
//!     .open()?;
//!
//! let hal = hal::FtHal::init_default(device)?;
//! let gpio = hal.ad6();
//! # }
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! ## More examples
//!
//! * [newAM/eeprom25aa02e48-rs]: read data from Microchip 25AA02E48 SPI EEPROM
//! * [newAM/bme280-rs]: read samples from Bosch BME280 sensor via I2C protocol
//!
//! [embedded-hal]: https://github.com/rust-embedded/embedded-hal
//! [ftdi-rs]: https://github.com/tanriol/ftdi-rs
//! [libftd2xx crate]: https://github.com/ftdi-rs/libftd2xx-rs/
//! [libftd2xx]: https://github.com/ftdi-rs/libftd2xx-rs
//! [newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/main/examples/ftdi.rs
//! [newAM/bme280-rs]: https://github.com/newAM/bme280-rs/blob/main/examples/ftdi-i2c.rs
//! [udev rules]: https://github.com/ftdi-rs/libftd2xx-rs/#udev-rules
//! [setup executable]: https://www.ftdichip.com/Drivers/CDM/CDM21228_Setup.zip
#![forbid(missing_docs)]
#![forbid(unsafe_code)]

pub use embedded_hal;
pub use ftdi_mpsse;

#[cfg(feature = "ftdi")]
pub use ftdi;

#[cfg(feature = "libftd2xx")]
pub use libftd2xx;

mod delay;
mod error;
mod gpio;
mod i2c;
mod spi;

use crate::error::Error;
pub use delay::Delay;
pub use gpio::{InputPin, OutputPin};
pub use i2c::I2c;
pub use spi::Spi;

use ftdi_mpsse::{MpsseCmdExecutor, MpsseSettings};
use std::sync::{Arc, Mutex};

/// State tracker for each pin on the FTDI chip.
#[derive(Debug, Clone, Copy)]
enum PinUse {
    I2c,
    Spi,
    Output,
    Input,
}

impl std::fmt::Display for PinUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PinUse::I2c => write!(f, "I2C"),
            PinUse::Spi => write!(f, "SPI"),
            PinUse::Output => write!(f, "OUTPUT"),
            PinUse::Input => write!(f, "INPUT"),
        }
    }
}

#[derive(Debug)]
struct FtInner<Device: MpsseCmdExecutor> {
    /// FTDI device.
    ft: Device,
    /// GPIO direction.
    direction: u8,
    /// GPIO value.
    value: u8,
    /// Pin allocation.
    pins: [Option<PinUse>; 8],
}

impl<Device: MpsseCmdExecutor> FtInner<Device> {
    /// Allocate a pin for a specific use.
    pub fn allocate_pin(&mut self, idx: u8, purpose: PinUse) {
        assert!(idx < 8, "Pin index {} is out of range 0 - 7", idx);

        if let Some(current) = self.pins[usize::from(idx)] {
            panic!(
                "Unable to allocate pin {} for {}, pin is already allocated for {}",
                idx, purpose, current
            );
        } else {
            self.pins[usize::from(idx)] = Some(purpose)
        }
    }
}

impl<Device: MpsseCmdExecutor> From<Device> for FtInner<Device> {
    fn from(ft: Device) -> Self {
        FtInner {
            ft,
            direction: 0x00,
            value: 0x00,
            pins: [None; 8],
        }
    }
}

/// FTxxx device.
#[derive(Debug)]
pub struct FtHal<Device: MpsseCmdExecutor> {
    mtx: Arc<Mutex<FtInner<Device>>>,
}

impl<Device, E> FtHal<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    /// Initialize the FTDI MPSSE with sane defaults.
    ///
    /// Default values:
    ///
    /// * Reset the FTDI device.
    /// * 4k USB transfer size.
    /// * 1s USB read timeout.
    /// * 1s USB write timeout.
    /// * 16ms latency timer.
    /// * 100kHz clock frequency.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
    /// let hal = hal::FtHal::init_default(device)?;
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn init_default(device: Device) -> Result<FtHal<Device>, Error<E>> {
        let settings: MpsseSettings = MpsseSettings {
            clock_frequency: Some(100_000),
            ..Default::default()
        };

        Ok(FtHal::init(device, &settings)?)
    }

    /// Initialize the FTDI MPSSE with sane defaults and custom frequency
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn init_freq(device: Device, freq: u32) -> Result<FtHal<Device>, Error<E>> {
        let settings: MpsseSettings = MpsseSettings {
            clock_frequency: Some(freq),
            ..Default::default()
        };

        Ok(FtHal::init(device, &settings)?)
    }

    /// Initialize the FTDI MPSSE with custom values.
    ///
    /// **Note:** The `mask` field of [`MpsseSettings`] is ignored for this function.
    ///
    /// **Note:** The clock frequency will be 2/3 of the specified value when in
    /// I2C mode.
    ///
    /// # Panics
    ///
    /// Panics if the `clock_frequency` field of [`MpsseSettings`] is `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    /// use ftdi_mpsse::MpsseSettings;
    /// use std::time::Duration;
    ///
    /// let mpsse = MpsseSettings {
    ///     reset: false,
    ///     in_transfer_size: 4096,
    ///     read_timeout: Duration::from_secs(5),
    ///     write_timeout: Duration::from_secs(5),
    ///     latency_timer: Duration::from_millis(32),
    ///     mask: 0x00,
    ///     clock_frequency: Some(400_000),
    /// };
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;
    /// let hal = hal::FtHal::init(device, &mpsse)?;
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`MpsseSettings`]: ftdi_mpsse::MpsseSettings
    pub fn init(mut device: Device, mpsse_settings: &MpsseSettings) -> Result<FtHal<Device>, E> {
        device.init(mpsse_settings)?;

        Ok(FtHal {
            mtx: Arc::new(Mutex::new(device.into())),
        })
    }
}

impl<Device, E> FtHal<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    /// Aquire the SPI peripheral for the FT232H.
    ///
    /// Pin assignments:
    /// * AD0 => SCK
    /// * AD1 => MOSI
    /// * AD2 => MISO
    ///
    /// # Panics
    ///
    /// Panics if pin 0, 1, or 2 are already in use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// let spi = hal.spi()?;
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn spi(&self) -> Result<Spi<Device>, Error<E>> {
        Spi::new(&self.mtx)
    }

    /// Aquire the I2C peripheral for the FT232H.
    ///
    /// Pin assignments:
    /// * AD0 => SCL
    /// * AD1 => SDA
    /// * AD2 => SDA
    ///
    /// Yes, AD1 and AD2 are both SDA.
    /// These pins must be shorted together for I2C operation.
    ///
    /// # Panics
    ///
    /// Panics if pin 0, 1, or 2 are already in use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// let i2c = hal.i2c()?;
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn i2c(&self) -> Result<I2c<Device>, Error<E>> {
        I2c::new(&self.mtx)
    }

    /// Aquire the digital output pin 0 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad0(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 0)
    }

    /// Aquire the digital input pin 0 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi0(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 0)
    }

    /// Aquire the digital output pin 1 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad1(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 1)
    }

    /// Aquire the digital input pin 1 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi1(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 1)
    }

    /// Aquire the digital output pin 2 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad2(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 2)
    }

    /// Aquire the digital input pin 2 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi2(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 2)
    }

    /// Aquire the digital output pin 3 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad3(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 3)
    }

    /// Aquire the digital input pin 3 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi3(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 3)
    }

    /// Aquire the digital output pin 4 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad4(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 4)
    }

    /// Aquire the digital input pin 4 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi4(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 4)
    }

    /// Aquire the digital output pin 5 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad5(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 5)
    }

    /// Aquire the digital input pin 5 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi5(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 5)
    }

    /// Aquire the digital output pin 6 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad6(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 6)
    }

    /// Aquire the digital input pin 6 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi6(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 6)
    }

    /// Aquire the digital output pin 7 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad7(&self) -> Result<OutputPin<Device>, Error<E>> {
        OutputPin::new(&self.mtx, 7)
    }

    /// Aquire the digital input pin 7 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi7(&self) -> Result<InputPin<Device>, Error<E>> {
        InputPin::new(&self.mtx, 7)
    }
}
