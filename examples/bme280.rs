//! This example reads the chip ID from a Bosch BME280.
//!
//! The hardware for this example can be purchased from adafruit:
//!
//! * https://www.adafruit.com/product/2264
//! * https://www.adafruit.com/product/2652
//! * https://www.adafruit.com/product/4399
//! * https://www.adafruit.com/product/4472

use embedded_hal::prelude::*;
use ftdi_embedded_hal as hal;

#[cfg(all(feature = "ftdi", feature = "libftd2xx"))]
compile_error!("features 'ftdi' and 'libftd2xx' cannot be enabled at the same time");

#[cfg(not(any(feature = "ftdi", feature = "libftd2xx")))]
compile_error!("one of features 'ftdi' and 'libftd2xx' shall be enabled");

fn main() {
    #[cfg(feature = "libftd2xx")]
    let device: libftd2xx::Ft232h = libftd2xx::Ftdi::new().unwrap().try_into().unwrap();

    #[cfg(feature = "ftdi")]
    let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(ftdi::Interface::A)
        .open()
        .unwrap();

    let hal = hal::FtHal::init_default(device).unwrap();
    let mut i2c = hal.i2c().unwrap();

    let mut buf: [u8; 1] = [0];
    const BME280_ADDR: u8 = 0b1110111;
    const BME280_CHIP_ID_ADDR: u8 = 0xD0;
    println!("Reading chip ID from BME280");
    i2c.write_read(BME280_ADDR, &[BME280_CHIP_ID_ADDR], &mut buf)
        .expect("Failed to read from BME280");

    // ID register is constant
    const BME280_CHIP_ID: u8 = 0x60;
    assert_eq!(buf[0], BME280_CHIP_ID);
    println!("Chip ID ok");
}
