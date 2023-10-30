# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added an I2C implementation for `embedded-hal` version 1.

### Changed
- Changed `ErrorKind::I2cNoAck` to have an inner type of `eh1::i2c::NoAcknowledgeSource`.

## [0.17.0] - 2023-08-15
### Changed
- Updated the alpha release of `embedded-hal` from `1.0.0-alpha.11` to `1.0.0-rc.1`.
- Updated the alpha release of `embedded-hal-nb` from `1.0.0-alpha.3` to `1.0.0-rc.1`.

## [0.16.0] - 2023-07-05
### Changed
- Updated the alpha release of `embedded-hal` from `1.0.0-alpha.10` to `1.0.0-alpha.11`.

## [0.15.1] - 2023-05-13
### Fixed
- Changed the `Error` and `ErrorKind` types from private to public.

## [0.15.0] - 2023-04-22
### Changed
- Updated the alpha release of `embedded-hal` from `1.0.0-alpha.9` to `1.0.0-alpha.10`.

## [0.14.0] - 2022-11-27
### Changed
- Removed lifetimes on `OutputPin`, `InputPin`, `I2c`, `Spi`, and `SpiDevice` to improve ease-of-use.

## [0.13.0] - 2022-09-28
### Changed
- Updated the alpha release of `embedded-hal` from `1.0.0-alpha.8` to `1.0.0-alpha.9`.

## [0.12.0] - 2022-09-03
### Added
- Added re-exports for `libftd2xx` and `ftdi` when the respective feature is used.
- Added `embedded-hal` version `1.0.0-alpha.8` trait implementations for:
  - GPIOs
  - Delay
  - SPI

### Changed
- Changed the `embedded-hal` version `0.2` re-export name from `embedded-hal` to
  `eh0` to differentiate from `embedded-hal` version `1.0.0-alpha.8`.

## [0.11.0] - 2022-01-18
### Added
- Added support for input pins.

### Changed
- The `ad0` - `ad7` methods to get an `OutputPin` now return a `Result` to
  support input pins, previously these methods were infallible.

## [0.10.0] - 2021-11-08
### Added
- Added support for `libftdi1` as a backend.

### Changed
- Renamed to `ftdi-embedded-hal`.
- The `ftd2xx` backend is no longer enabled by default.
- Changed the error type to support multiple backends.
- Updated the edition from 2018 to 2021.

## [0.9.1] - 2021-08-10
### Fixed
- Call `close()` on Drop, allowing recovery from failures in `init()`.

## [0.9.0] - 2021-07-01
### Changed
- Updated the `libftd2xx` dependency from 0.29.0 to 0.31.0.

## [0.8.0] - 2021-05-29
### Changed
- Updated the `libftd2xx` dependency from 0.28.0 to 0.29.0.

## [0.7.0] - 2021-04-18
### Added
- Added checks for missing ACKs from the I2C slave.
  Missing ACKs will now return an `NakError` from the I2C traits.

### Changed
- Changed the default implementation of I2C traits to wait for a slave ACK
  before transmitting more bytes.  The previous behavior can be retained by
  calling `set_fast(true)`.

## [0.6.0] - 2021-04-10
### Added
- Added support for the FT4232H.
- Added support for the FT2232H.

### Changed
- Changed the default linking method on Linux to dynamic.
  Static linking can be enabled with the `static` feature flag.
- Changed the I2C pins to input (tri-state) when in idle mode.

### Fixed
- Fixed AD0 (SCL) pulling low when when I2C is first initialized.
- Fixed I2C AD0 & AD1 (SCL & SDA out) being pulled low when another OutputPin
  changed state.

## [0.5.1] - 2021-03-20
### Fixed
- Fixed the I2C `Read` trait not setting the read address bit.
- Fixed the I2C `Write` trait not driving SDA as an output when clocking data
  out.

## [0.5.0] - 2021-03-20
### Added
- Added checks for pin allocation, trying to take output pins 0-2 while using
  the SPI interface will now result in panic.
- Added I2C traits.
- Added `Debug` for interface structures.
- Added `with_serial_number` and `with_description` constructors.

### Changed
- Changed the FTDI MPSSE initialization to occur once globally for the device
  instead of inside the SPI device trait.
- Changed the `Delay` structure to contain dummy data for possible future use.
- Change the `Ft232hHal::with_ft` to `impl From<Ft232h> for Ft232hHal`.

### Removed
- Removed `Eq` and `PartialEq` traits on the `Delay` structure.

## [0.4.0] - 2021-03-05
### Added
- Added a `Delay` structure that implements the embedded-hal delay traits.

### Changed
- Updated `libftd2xx` dependency from 0.24.0 to 0.25.0.
  This updates the vendor library from 1.4.8 to 1.4.22 for Linux targets.
  This should fix timeout conditions that previously occurred when rapidly
  toggling GPIO pins.

## [0.3.0] - 2021-02-14
### Changed
- Improved latency for GPIOs

### Fixed
- Fixed the example code for `Ft232hHal::new`.
- Fixed pins 5, 6, 7 not being usable as outputs.

## [0.2.0] - 2020-09-13
### Added
- Added SPI non-blocking traits.

### Changed
- Updated to libftd2xx 0.17.0
- Updated to embedded-hal 0.2.4

## [0.1.0] - 2020-09-12
- Initial release

[Unreleased]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.17.0...HEAD
[0.17.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.16.0...v0.17.0
[0.16.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.15.1...v0.16.0
[0.15.1]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.15.0...v0.15.1
[0.15.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.14.0...v0.15.0
[0.14.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ftdi-rs/ftdi-embedded-hal/releases/tag/v0.1.0
