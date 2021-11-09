use embedded_hal::digital::v2::OutputPin;
use ftdi_embedded_hal as hal;
use std::{thread::sleep, time::Duration};

const NUM_BLINK: usize = 10;
const SLEEP_DURATION: Duration = Duration::from_millis(500);

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
    let mut output_pin = hal.ad3();

    println!("Starting blinky example");
    for n in 0..NUM_BLINK {
        output_pin.set_high().expect("failed to set GPIO");
        sleep(SLEEP_DURATION);
        output_pin.set_low().expect("failed to set GPIO");
        sleep(SLEEP_DURATION);
        println!("Blinked {}/{} times", n + 1, NUM_BLINK);
    }
}
