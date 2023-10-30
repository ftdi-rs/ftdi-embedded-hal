//! This is a loopback example for embedded-hal-1 traits:
//! * SpiBus
//! * SpiDevice (TODO)
//! Pin setup:
//! * D1 <-> D2 (connect SDO with SDI)
//! Leave other pins unconnected.
use ftdi_embedded_hal as hal;
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

    let hal = hal::FtHal::init_freq(device, 1_000_000).unwrap();
    let mut spi = hal.spi().unwrap();

    let delay = Duration::from_millis(250);

    {
        use hal::eh1::spi::SpiBus;

        println!("=== SpiBus example with embedded-hal-1 traits ===");

        // --- Symmetric transfer (Read as much as we write) ---
        print!("Starting symmetric transfer...");
        let write = [0xde, 0xad, 0xbe, 0xef];
        let mut read: [u8; 4] = [0x00u8; 4];
        SpiBus::transfer(&mut spi, &mut read[..], &write[..]).expect("Symmetric transfer failed");
        assert_eq!(write, read);
        println!(" SUCCESS");

        // XXX: After we do an asymmetric transfer, we still have two leftover
        // bytes are still in the read buffer, which breaks tests afterwards.
        // Spi::flush(&mut spi) doesn't help either

        // --- Asymmetric transfer (Read more than we write) ---
        print!("Starting asymetric transfer (read > write)...");
        let mut read: [u8; 4] = [0x00; 4];

        SpiBus::transfer(&mut spi, &mut read[0..2], &write[..])
            .expect("Asymmetric transfer failed");
        assert_eq!(write[0], read[0]);
        assert_eq!(read[2], 0x00u8);
        println!(" SUCCESS");
        sleep(delay);

        // --- Symmetric transfer with huge buffer ---
        // Only your RAM is the limit!
        print!("Starting huge transfer...");
        let mut write = [0x55u8; 4096];
        for byte in 0..write.len() {
            write[byte] = byte as u8;
        }
        let mut read = [0x00u8; 4096];
        sleep(delay);

        SpiBus::transfer(&mut spi, &mut read[..], &write[..]).expect("Huge transfer failed");
        assert_eq!(write, read);
        println!(" SUCCESS");
        sleep(delay);

        // --- Symmetric transfer with huge buffer in-place (No additional allocation
        // needed) ---
        print!("Starting huge transfer (in-place)...");
        let mut write = [0x55u8; 4096];
        for byte in 0..write.len() {
            write[byte] = byte as u8;
        }

        SpiBus::transfer_in_place(&mut spi, &mut write[..]).expect("Huge transfer failed");
        for byte in 0..write.len() {
            assert_eq!(write[byte], byte as u8);
        }
        println!(" SUCCESS");
        sleep(delay);
    }

    // TODO: eh1 SpiDevice
}
