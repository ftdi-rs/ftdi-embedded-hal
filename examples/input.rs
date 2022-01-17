#[cfg(feature = "unproven")]
use embedded_hal::digital::v2::InputPin;
use ftdi_embedded_hal as hal;
use std::{thread::sleep, time::Duration};

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

    let _hal = hal::FtHal::init_default(device).unwrap();
    #[cfg(feature = "unproven")]
    let input = _hal.adi6().unwrap();

    println!("Pin readings:");
    loop {
        #[cfg(feature = "unproven")]
        println!("AD6 = {}", input.is_high().expect("gpio read failure"));
        #[cfg(not(feature = "unproven"))]
        println!("Use 'unproven' feature to enable input gpio");
        sleep(SLEEP_DURATION);
    }
}
