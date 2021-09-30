//! Inspired by [ftdi-embedded-hal] this is an [embedded-hal] implementation
//! for the for the FTDI chips using the [libftd2xx] drivers.
//!
//! This enables development of embedded device drivers without the use of a
//! microcontroller.
//! The FTDI D2xx devices interface with your PC via USB, and provide a
//! multi-protocol synchronous serial engine to interface with most UART, SPI,
//! and I2C embedded devices.
//!
//! **Note:**
//! This is strictly a development tool.
//! The crate contains runtime borrow checks and explicit panics to adapt the
//! FTDI device into the [embedded-hal] traits.
//!
//! # Quickstart
//!
//! * Enable the "static" feature flag to use static linking.
//! * Linux users only: Add [udev rules].
//!
//! ```toml
//! [dependencies.ftdi-embedded-hal]
//! version = "~0.9.1"
//! features = ["static"]
//! ```
//!
//! # Examples
//!
//! * [newAM/eeprom25aa02e48-rs]
//! * [newAM/bme280-rs]
//!
//! ## SPI
//!
//! ```no_run
//! use embedded_hal::prelude::*;
//! use ftdi_embedded_hal::Ft232hHal;
//!
//! let ftdi = Ft232hHal::new()?.init_default()?;
//! let mut spi = ftdi.spi()?;
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! ## I2C
//!
//! ```no_run
//! use embedded_hal::prelude::*;
//! use ftdi_embedded_hal::Ft232hHal;
//!
//! let ftdi = Ft232hHal::new()?.init_default()?;
//! let mut i2c = ftdi.i2c()?;
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! ## GPIO
//!
//! ```no_run
//! use embedded_hal::prelude::*;
//! use ftdi_embedded_hal::Ft232hHal;
//!
//! let ftdi = Ft232hHal::new()?.init_default()?;
//! let mut gpio = ftdi.ad6();
//! # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
//! ```
//!
//! # Limitations
//!
//! * Limited trait support: SPI, I2C, Delay, and OutputPin traits are implemented.
//! * Limited device support: FT232H, FT2232H, FT4232H.
//!
//! [embedded-hal]: https://github.com/rust-embedded/embedded-hal
//! [ftdi-embedded-hal-archive]: https://github.com/geomatsi/ftdi-embedded-hal-archive
//! [libftd2xx crate]: https://github.com/ftdi-rs/libftd2xx-rs/
//! [libftd2xx]: https://github.com/ftdi-rs/libftd2xx-rs
//! [newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/main/examples/ftdi.rs
//! [newAM/bme280-rs]: https://github.com/newAM/bme280-rs/blob/main/examples/ftdi.rs
//! [udev rules]: https://github.com/ftdi-rs/libftd2xx-rs/#udev-rules
//! [setup executable]: https://www.ftdichip.com/Drivers/CDM/CDM21228_Setup.zip
#![doc(html_root_url = "https://docs.rs/ftdi-embedded-hal/0.9.1")]
#![forbid(missing_docs)]
#![forbid(unsafe_code)]

pub use embedded_hal;
pub use ftdi_mpsse;

mod delay;
mod gpio;
mod i2c;
mod spi;

pub use delay::Delay;
pub use gpio::OutputPin;
pub use i2c::{I2c, I2cError};
pub use spi::Spi;

use ftdi_mpsse::{MpsseSettings, MpsseCmdExecutor};
use std::{cell::RefCell, sync::Mutex};

/// State tracker for each pin on the FTDI chip.
#[derive(Debug, Clone, Copy)]
enum PinUse {
    I2c,
    Spi,
    Output,
}

impl std::fmt::Display for PinUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PinUse::I2c => write!(f, "I2C"),
            PinUse::Spi => write!(f, "SPI"),
            PinUse::Output => write!(f, "GPIO"),
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

impl<Device: FtdiCommon> Drop for FtInner<Device> {
    fn drop(&mut self) {
        self.ft.close().ok();
    }
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
            direction: 0xFB,
            value: 0x00,
            pins: [None; 8],
        }
    }
}

/// Type state for an initialized FTDI HAL.
///
/// More information about type states can be found in the [rust-embedded book].
///
/// [rust-embedded book]: https://docs.rust-embedded.org/book/static-guarantees/design-contracts.html
pub struct Initialized;

/// Type state for an uninitialized FTDI HAL.
///
/// More information about type states can be found in the [rust-embedded book].
///
/// [rust-embedded book]: https://docs.rust-embedded.org/book/static-guarantees/design-contracts.html
pub struct Uninitialized;

/// FTxxx device.
#[derive(Debug)]
pub struct FtHal<Device: MpsseCmdExecutor, INITIALIZED> {
    #[allow(dead_code)]
    init: INITIALIZED,
    mtx: Mutex<RefCell<FtInner<Device>>>,
}

impl<Device: MpsseCmdExecutor> FtHal<Device, Uninitialized>
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
    /// use hal::{Ft232hHal, Initialized, Uninitialized};
    ///
    /// let ftdi: Ft232hHal<Uninitialized> = hal::Ft232hHal::new()?;
    /// let ftdi: Ft232hHal<Initialized> = ftdi.init_default()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn init_default(device: Device) -> Result<FtHal<Device, Initialized>> {
        FtHal::init(device, &MpsseSettings::default())
    }
    pub fn init_basic(device: Device, freq: u32) -> Result<FtHal<Device, Initialized>> {
        let settings: MpsseSettings = MpsseSettings {
            clock_frequency: Some(freq),
            ..Default::default()
        };

        FtHal::init(device, &settings)
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
    /// use hal::libftd2xx::MpsseSettings;
    /// use hal::{Ft232hHal, Initialized, Uninitialized};
    ///
    /// let ftdi: Ft232hHal<Uninitialized> = hal::Ft232hHal::new()?;
    /// let ftdi: Ft232hHal<Initialized> = ftdi.init(&MpsseSettings {
    ///     clock_frequency: Some(500_000),
    ///     ..MpsseSettings::default()
    /// })?;
    ///
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`MpsseSettings`]: libftd2xx::MpsseSettings
    pub fn init(
        mut device: Device,
        mpsse_settings: &MpsseSettings,
    ) -> Result<FtHal<Device, Initialized>> {
        device.init(mpsse_settings)?;

        Ok(FtHal {
            init: Initialized,
            mtx: Mutex::new(RefCell::new(device.into()))
        })
    }
}

impl<Device: MpsseCmdExecutor> From<Device> for FtHal<Device, Uninitialized>
{
    /// Create a new FT232H structure from a specific FT232H device.
    ///
    /// # Examples
    ///
    /// Selecting a device with a specific serial number.
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    /// use hal::libftd2xx::Ft232h;
    /// use hal::Ft232hHal;
    ///
    /// let ft = Ft232h::with_serial_number("FT59UO4C")?;
    /// let ftdi = Ft232hHal::from(ft).init_default()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Selecting a device with a specific description.
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    /// use hal::libftd2xx::Ft232h;
    /// use hal::FtHal;
    ///
    /// let ft = Ft232h::with_description("My device description")?;
    /// let ftdi = FtHal::from(ft).init_default()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    fn from(ft: Device) -> Self {
        FtHal {
            init: Uninitialized,
            mtx: Mutex::new(RefCell::new(ft.into())),
        }
    }
}

impl<Device: MpsseCmdExecutor> FtHal<Device, Initialized>
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
    /// let ftdi = hal::Ft232hHal::new()?.init_default()?;
    /// let mut spi = ftdi.spi()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn spi(&self) -> Result<Spi<Device>, TimeoutError> {
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
    /// let ftdi = hal::Ft232hHal::new()?.init_default()?;
    /// let mut i2c = ftdi.i2c()?;
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn i2c(&self) -> Result<I2c<Device>, TimeoutError> {
        I2c::new(&self.mtx)
    }

    /// Aquire the digital output pin 0 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad0(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 0)
    }

    /// Aquire the digital output pin 1 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad1(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 1)
    }

    /// Aquire the digital output pin 2 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad2(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 2)
    }

    /// Aquire the digital output pin 3 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad3(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 3)
    }

    /// Aquire the digital output pin 4 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad4(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 4)
    }

    /// Aquire the digital output pin 5 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad5(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 5)
    }

    /// Aquire the digital output pin 6 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad6(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 6)
    }

    /// Aquire the digital output pin 7 for the FT232H.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad7(&self) -> OutputPin<Device> {
        OutputPin::new(&self.mtx, 7)
    }
}
