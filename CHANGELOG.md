# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added support for the FT4232H.
- Added support for the FT2232H.

### Changed
- Changed the default linking method on Linux to dynamic.
  Static linking can be enabled with the `static` feature flag.

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

[Unreleased]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/newAM/ftd2xx-embedded-hal/releases/tag/v0.1.0
