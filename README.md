![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/ftd2xx-embedded-hal.svg)](https://crates.io/crates/ftd2xx-embedded-hal)
[![docs.rs](https://docs.rs/ftd2xx-embedded-hal/badge.svg)](https://docs.rs/ftd2xx-embedded-hal/)
[![Build Status](https://github.com/newAM/ftd2xx-embedded-hal/workflows/CI/badge.svg)](https://github.com/newAM/ftd2xx-embedded-hal/actions)

# ftd2xx-embedded-hal

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

## One-time Setup

Detailed setup instructions can be found in the [libftd2xx crate].

### Linux Quickstart

Use static linking (enable the "static" feature flag), and add [udev rules].

```toml
[dependencies.ftd2xx-embedded-hal]
version = "~0.5.1"
features = ["static"]
```

### Windows Quickstart

Use dyanmic linking (no feature flags), and run the vendor
[setup executable] to install the vendor library on your system.

```toml
[dependencies.ftd2xx-embedded-hal]
version = "~0.5.1"
```

## Examples

* [newAM/eeprom25aa02e48-rs]
* [newAM/bme280-rs]

### SPI

```rust
use embedded_hal::prelude::*;
use ftd2xx_embedded_hal::Ft232hHal;

let ftdi = Ft232hHal::new()?.init_default()?;
let mut spi = ftdi.spi()?;
```

### I2C

```rust
use embedded_hal::prelude::*;
use ftd2xx_embedded_hal::Ft232hHal;

let ftdi = Ft232hHal::new()?.init_default()?;
let mut i2c = ftdi.i2c()?;
```

## GPIO

```rust
use embedded_hal::prelude::*;
use ftd2xx_embedded_hal::Ft232hHal;

let ftdi = Ft232hHal::new()?.init_default()?;
let mut gpio = ftdi.ad6();
```

## Limitations

* Limited trait support: SPI, I2C, Delay, and OutputPin traits are implemented.
* Limited device support: FT232H, FT4232H.

[embedded-hal]: https://github.com/rust-embedded/embedded-hal
[ftdi-embedded-hal]: https://github.com/geomatsi/ftdi-embedded-hal
[libftd2xx crate]: https://github.com/newAM/libftd2xx-rs/
[libftd2xx]: https://github.com/newAM/libftd2xx-rs
[newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/main/examples/ftdi.rs
[newAM/bme280-rs]: https://github.com/newAM/bme280-rs/blob/main/examples/ftdi.rs
[udev rules]: https://github.com/newAM/libftd2xx-rs/#udev-rules
[setup executable]: https://www.ftdichip.com/Drivers/CDM/CDM21228_Setup.zip
