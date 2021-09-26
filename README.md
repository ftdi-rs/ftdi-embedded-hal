![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/ftdi-embedded-hal.svg)](https://crates.io/crates/ftdi-embedded-hal)
[![docs.rs](https://docs.rs/ftdi-embedded-hal/badge.svg)](https://docs.rs/ftdi-embedded-hal/)
[![Build Status](https://github.com/ftdi-rs/ftdi-embedded-hal/workflows/CI/badge.svg)](https://github.com/ftdi-rs/ftdi-embedded-hal/actions)

# ftdi-embedded-hal

Inspired by [ftdi-embedded-hal] this is an [embedded-hal] implementation
for the for the FTDI chips using the [libftd2xx] drivers.

This enables development of embedded device drivers without the use of a
microcontroller.
The FTDI D2xx devices interface with your PC via USB, and provide a
multi-protocol synchronous serial engine to interface with most UART, SPI,
and I2C embedded devices.

**Note:**
This is strictly a development tool.
The crate contains runtime borrow checks and explicit panics to adapt the
FTDI device into the [embedded-hal] traits.

## Quickstart

* Enable the "static" feature flag to use static linking.
* Linux users only: Add [udev rules].

```toml
[dependencies.ftdi-embedded-hal]
version = "~0.9.1"
features = ["static"]
```

## Examples

* [newAM/eeprom25aa02e48-rs]
* [newAM/bme280-rs]

### SPI

```rust
use embedded_hal::prelude::*;
use ftdi_embedded_hal::Ft232hHal;

let ftdi = Ft232hHal::new()?.init_default()?;
let mut spi = ftdi.spi()?;
```

### I2C

```rust
use embedded_hal::prelude::*;
use ftdi_embedded_hal::Ft232hHal;

let ftdi = Ft232hHal::new()?.init_default()?;
let mut i2c = ftdi.i2c()?;
```

### GPIO

```rust
use embedded_hal::prelude::*;
use ftdi_embedded_hal::Ft232hHal;

let ftdi = Ft232hHal::new()?.init_default()?;
let mut gpio = ftdi.ad6();
```

## Limitations

* Limited trait support: SPI, I2C, Delay, and OutputPin traits are implemented.
* Limited device support: FT232H, FT2232H, FT4232H.

[embedded-hal]: https://github.com/rust-embedded/embedded-hal
[ftdi-embedded-hal-archive]: https://github.com/geomatsi/ftdi-embedded-hal-archive
[libftd2xx crate]: https://github.com/ftdi-rs/libftd2xx-rs/
[libftd2xx]: https://github.com/ftdi-rs/libftd2xx-rs
[newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/main/examples/ftdi.rs
[newAM/bme280-rs]: https://github.com/newAM/bme280-rs/blob/main/examples/ftdi.rs
[udev rules]: https://github.com/ftdi-rs/libftd2xx-rs/#udev-rules
[setup executable]: https://www.ftdichip.com/Drivers/CDM/CDM21228_Setup.zip
