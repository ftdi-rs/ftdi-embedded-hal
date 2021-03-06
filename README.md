![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/ftd2xx-embedded-hal.svg)](https://crates.io/crates/ftd2xx-embedded-hal)
[![docs.rs](https://docs.rs/ftd2xx-embedded-hal/badge.svg)](https://docs.rs/ftd2xx-embedded-hal/)
[![Build Status](https://github.com/newAM/ftd2xx-embedded-hal/workflows/CI/badge.svg)](https://github.com/newAM/ftd2xx-embedded-hal/actions)

# ftd2xx-embedded-hal

Inspired by [ftdi-embedded-hal] this is an [embedded-hal] implementation
for the for the FTDI chips using the [libftd2xx] drivers.

This enables development of embedded devices drivers without the use of a
microcontroller.
The FTDI 2xx devices interface with your PC via USB.
They have a multi-protocol synchronous serial engine which allows them to
interface with most UART, SPI, and I2C embedded devices.

**Note:**
This is strictly a development tool.
The crate contains runtime borrow checks and explicit panics to adapt the
FTDI device into the [embedded-hal] traits.

## Setup

One-time device setup instructions can be found in the [libftd2xx crate].

## Examples

* [newAM/eeprom25aa02e48-rs]

## Limitations

* Limited trait support: SPI and OutputPin traits are implemented.
* Limited device support: FT232H.

[embedded-hal]: https://crates.io/crates/embedded-hal
[ftdi-embedded-hal]: https://github.com/geomatsi/ftdi-embedded-hal
[libftd2xx crate]: https://github.com/newAM/libftd2xx-rs/
[libftd2xx]: https://github.com/newAM/libftd2xx-rs
[newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/master/examples/ftdi.rs
