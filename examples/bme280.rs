//! This example reads the chip ID from a Bosch BME280.
//!
//! The hardware for this example can be purchased from adafruit:
//!
//! * https://www.adafruit.com/product/2264
//! * https://www.adafruit.com/product/2652
//! * https://www.adafruit.com/product/4399
//! * https://www.adafruit.com/product/4472

use embedded_hal::prelude::*;
use ftd2xx_embedded_hal as hal;

fn main() {
    let ftdi = hal::Ft232hHal::new()
        .expect("Failed to open FT232H device")
        .init_default()
        .expect("Failed to initialize MPSSE");
    let mut i2c = ftdi.i2c().expect("Failed to initialize I2C");

    let mut buf: [u8; 1] = [0];
    const BME280_ADDR: u8 = 0b1110111;
    const BME280_CHIP_ID_ADDR: u8 = 0xD0;
    i2c.write_read(BME280_ADDR, &[BME280_CHIP_ID_ADDR], &mut buf)
        .expect("Failed to read from BME280");

    // ID register is constant
    const BME280_CHIP_ID: u8 = 0x60;
    assert_eq!(buf[0], BME280_CHIP_ID);
}
