use ftdi_embedded_hal as hal;
use lm75::{Address, Lm75};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ftdi")] {
            let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
            .interface(ftdi::Interface::A)
            .open()
            .unwrap();
        } else if #[cfg(feature = "libftd2xx")] {
            let device = libftd2xx::Ft232h::with_description("Single RS232-HS").unwrap();
        } else {
            compile_error!("one of features 'ftdi' and 'libftd2xx' shall be enabled");
        }
    }

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
