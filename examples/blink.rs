use embedded_hal::digital::v2::OutputPin;
use ftd2xx_embedded_hal as hal;
use std::{thread::sleep, time::Duration};

const NUM_BLINK: usize = 10;
const SLEEP_DURATION: Duration = Duration::from_millis(500);

fn main() {
    let ftdi = hal::Ft232hHal::new()
        .expect("Failed to open FT232H device")
        .init_default()
        .expect("Failed to initialize MPSSE");
    let mut output_pin = ftdi.ad3();

    println!("Starting blinky example");
    for n in 0..NUM_BLINK {
        output_pin.set_high().expect("failed to set GPIO");
        sleep(SLEEP_DURATION);
        output_pin.set_low().expect("failed to set GPIO");
        sleep(SLEEP_DURATION);
        println!("Blinked {}/{} times", n + 1, NUM_BLINK);
    }
}
