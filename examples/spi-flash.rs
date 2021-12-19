use ftdi_embedded_hal as hal;
use spi_memory::prelude::*;
use spi_memory::series25::Flash;
use std::thread::sleep;
use std::time::Duration;

#[cfg(all(feature = "ftdi", feature = "libftd2xx"))]
compile_error!("features 'ftdi' and 'libftd2xx' cannot be enabled at the same time");

#[cfg(not(any(feature = "ftdi", feature = "libftd2xx")))]
compile_error!("one of features 'ftdi' and 'libftd2xx' shall be enabled");

const LINE: u32 = 0x10;

fn main() {
    #[cfg(feature = "ftdi")]
    let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
        .interface(ftdi::Interface::A)
        .open()
        .unwrap();

    #[cfg(feature = "libftd2xx")]
    let device = libftd2xx::Ft232h::with_description("Single RS232-HS").unwrap();

    let hal = hal::FtHal::init_freq(device, 1_000_000).unwrap();
    let spi = hal.spi().unwrap();
    let ncs = hal.ad3().unwrap();
    let delay = Duration::from_millis(10);

    let mut flash = Flash::init(spi, ncs).unwrap();
    let id = flash.read_jedec_id().unwrap();
    println!("JEDEC ID: {:?}", id);

    #[cfg(feature = "libftd2xx")]
    let data: [u8; 8] = [0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70];
    #[cfg(feature = "ftdi")]
    let data: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];

    let addrs: [u32; 5] = [0, LINE, 2 * LINE, 3 * LINE, 4 * LINE];
    let zero: [u8; 8] = [0; 8];
    let mut bytes_w: [u8; 8] = [0; 8];
    let mut bytes_r: [u8; 8] = [0; 8];

    println!("Write to flash...");
    for addr in addrs.iter() {
        bytes_w.copy_from_slice(&data);
        println!("Write bytes {:02x?} to address {:02x}", bytes_w, *addr);
        flash.write_bytes(*addr, &mut bytes_w).unwrap();
        sleep(delay);
    }

    println!("Read from flash and check...");
    for addr in addrs.iter() {
        bytes_r.copy_from_slice(&zero);
        flash.read(*addr, &mut bytes_r).unwrap();
        println!("Read byte from address {:02x}: {:02x?}", *addr, bytes_r);
        assert_eq!(data, bytes_r);
        sleep(delay);
    }

    let mut buf = [0; LINE as usize];
    let mut addr = 0;
    println!("Dump flash...");
    while addr < 0x100 {
        flash.read(addr, &mut buf).unwrap();
        println!("{:02x}: {:02x?}", addr, buf);
        addr += LINE as u32;
    }
}
