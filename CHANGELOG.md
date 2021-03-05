# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2021-03-05
### Added
- Added a `Delay` structure that implements the embedded-hal delay traits.

### Changed
- Updated `libftd2xx` dependency from 0.24.0 to 0.25.0.  This updates the vendor library from 1.4.8 to 1.4.22 for Linux targets.  This should fix timeout conditions that previously occurred when rapidly toggling GPIO pins.

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

[Unreleased]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/newAM/ftd2xx-embedded-hal/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/newAM/ftd2xx-embedded-hal/releases/tag/v0.1.0
