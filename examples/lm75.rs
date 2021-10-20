use ftdi_embedded_hal as hal;
use lm75::{Address, Lm75};
use std::thread::sleep;
use std::time::Duration;

#[cfg(all(feature = "ftdi", feature = "libftd2xx"))]
compile_error!("features 'ftdi' and 'libftd2xx' cannot be enabled at the same time");

#[cfg(not(any(feature = "ftdi", feature = "libftd2xx")))]
compile_error!("one of features 'ftdi' and 'libftd2xx' shall be enabled");

fn main() {
    #[cfg(feature = "ftdi")]
    let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(ftdi::Interface::A)
        .open()
        .unwrap();

    #[cfg(feature = "libftd2xx")]
    let device = libftd2xx::Ft232h::with_description("Single RS232-HS").unwrap();

    let hal = hal::FtHal::init_freq(device, 400_000).unwrap();
    let i2c = hal.i2c().unwrap();
    let mut sensor = Lm75::new(i2c, Address::default());
    let delay = Duration::from_secs(1);

    loop {
        let temperature = sensor.read_temperature().unwrap();
        println!("Temperature: {}", temperature);
        sleep(delay);
    }
}
