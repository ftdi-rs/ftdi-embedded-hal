![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)
[![crates.io](https://img.shields.io/crates/v/ftdi-embedded-hal.svg)](https://crates.io/crates/ftdi-embedded-hal)
[![docs.rs](https://docs.rs/ftdi-embedded-hal/badge.svg)](https://docs.rs/ftdi-embedded-hal/)
[![Build Status](https://github.com/ftdi-rs/ftdi-embedded-hal/workflows/CI/badge.svg)](https://github.com/ftdi-rs/ftdi-embedded-hal/actions)

# ftdi-embedded-hal

This is an [embedded-hal] implementation for the FTDI chips
that can use various drivers including [libftd2xx] and [ftdi-rs].

This enables development of embedded device drivers without the use of
a microcontroller. The FTDI devices interface with PC via USB, and
provide a multi-protocol synchronous serial engine to interface
with most GPIO, SPI, I2C embedded devices.

**Note:**
This is strictly a development tool.
The crate contains runtime borrow checks and explicit panics to adapt the
FTDI device into the [embedded-hal] traits.

## Quickstart

* Enable the "libftd2xx-static" feature flag to use static linking with libftd2xx driver.
* Linux users only: Add [udev rules].

```toml
[dependencies.ftdi-embedded-hal]
version = "0.22.1"
features = ["libftd2xx", "libftd2xx-static"]
```

## Limitations

* Limited trait support: SPI, I2C, Delay, InputPin, and OutputPin traits are implemented.
* Limited device support: FT232H, FT2232H, FT4232H.
* Limited SPI modes support: MODE0, MODE2.

## Examples

### SPI

Pin setup:

* D0 - SCK
* D1 - SDO (MOSI)
* D2 - SDI (MISO)
* D3..D7 - Available for CS

Communicate with SPI devices using [ftdi-rs] driver:
```rust
use ftdi_embedded_hal as hal;

let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
    .interface(ftdi::Interface::A)
    .open()?;

let hal = hal::FtHal::init_freq(device, 3_000_000)?;
let spi = hal.spi()?;
```

Communicate with SPI devices using [libftd2xx] driver:
```rust
use ftdi_embedded_hal as hal;

let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;

let hal = hal::FtHal::init_freq(device, 3_000_000)?;
let spi = hal.spi()?;
```

### I2C

Communicate with I2C devices using [ftdi-rs] driver:
```rust
use ftdi_embedded_hal as hal;

let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
    .interface(ftdi::Interface::A)
    .open()?;

let hal = hal::FtHal::init_freq(device, 400_000)?;
let i2c = hal.i2c()?;
```

Communicate with I2C devices using [libftd2xx] driver:
```rust
use ftdi_embedded_hal as hal;

let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;

let hal = hal::FtHal::init_freq(device, 400_000)?;
let i2c = hal.i2c()?;
```

### GPIO

Control GPIO pins using [libftd2xx] driver:
```rust
use ftdi_embedded_hal as hal;

let device = libftd2xx::Ft232h::with_description("Single RS232-HS")?;

let hal = hal::FtHal::init_default(device)?;
let gpio = hal.ad6();
```

Control GPIO pins using [ftdi-rs] driver:
```rust
use ftdi_embedded_hal as hal;

let device = ftdi::find_by_vid_pid(0x0403, 0x6010)
    .interface(ftdi::Interface::A)
    .open()?;

let hal = hal::FtHal::init_default(device)?;
let gpio = hal.ad6();
```

### More examples

* [newAM/eeprom25aa02e48-rs]: read data from Microchip 25AA02E48 SPI EEPROM
* [newAM/bme280-rs]: read samples from Bosch BME280 sensor via I2C protocol

[embedded-hal]: https://github.com/rust-embedded/embedded-hal
[ftdi-rs]: https://github.com/tanriol/ftdi-rs
[libftd2xx crate]: https://github.com/ftdi-rs/libftd2xx-rs/
[libftd2xx]: https://github.com/ftdi-rs/libftd2xx-rs
[newAM/eeprom25aa02e48-rs]: https://github.com/newAM/eeprom25aa02e48-rs/blob/main/examples/ftdi.rs
[newAM/bme280-rs]: https://github.com/newAM/bme280-rs/blob/main/examples/ftdi-i2c.rs
[udev rules]: https://github.com/ftdi-rs/libftd2xx-rs/#udev-rules
[setup executable]: https://www.ftdichip.com/Drivers/CDM/CDM21228_Setup.zip
