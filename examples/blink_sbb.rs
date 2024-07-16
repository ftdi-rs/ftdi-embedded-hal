use eh0::digital::v2::OutputPin;
use ftdi_embedded_hal as hal;
use std::{thread::sleep, time::Duration};

const NUM_BLINK: usize = 10;
const SLEEP_DURATION: Duration = Duration::from_millis(500);

/// Toggle the AD0 output by reading its state on AD1, inverting, and
/// writing it back out on AD0. This test requires that AD0 and AD1 are
/// connected together.
fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ftdi")] {
            // FTDI FT4232H: vid=0x0403, pid=0x6011.
            let device = ftdi::find_by_vid_pid(0x0403, 0x6011)
            .interface(ftdi::Interface::D)
            .open()
            .unwrap();

            // Default settings suffice.
            let hal_cfg = hal::FtHalSbbSettings::default();
            let hal = hal::FtHalSbb::init(epe_if_d, hal_cfg).unwrap();

            // Assign the GPIO pins.
            let gpio_ado0 = hal.ad0().unwrap();
            let gpio_adi1 = hal.adi1().unwrap();

            println!("Starting blinky using synchronous bit-bang gpio example");
            for n in 0..NUM_BLINK {
                let state = gpio_adi1.get().expect("failed to get GPIO AD1");
                println!("Read State: {}", state);

                gpio_ado0.set(!state).expect("failed to set GPIO AD0");

                sleep(SLEEP_DURATION);

                println!("Blinked {}/{} times", n + 1, NUM_BLINK);
            }

        } else {
            compile_error!("Feature 'ftdi' must be enabled");
        }
    }
}
