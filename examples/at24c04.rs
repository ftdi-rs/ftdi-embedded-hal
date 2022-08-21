use eeprom24x::Eeprom24x;
use eeprom24x::SlaveAddr;
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

    let hal = hal::FtHal::init_freq(device, 400_000).unwrap();
    let i2c = hal.i2c().unwrap();
    let mut eeprom = Eeprom24x::new_24x04(i2c, SlaveAddr::default());
    let delay = Duration::from_millis(5);

    // check high memory addresses: 1 bit passed as a part of i2c addr
    let addrs1: [u32; 4] = [0x100, 0x10F, 0x1F0, 0x1EE];
    let byte_w1 = 0xe5;
    let addrs2: [u32; 4] = [0x00, 0x0F, 0xF0, 0xEE];
    let byte_w2 = 0xaa;

    // write bytes

    for addr in addrs1.iter() {
        println!("Write byte {:#x} to address {:#x}", byte_w1, *addr);
        eeprom.write_byte(*addr, byte_w1).unwrap();
        sleep(delay);
    }

    for addr in addrs2.iter() {
        println!("Write byte {:#x} to address {:#x}", byte_w2, *addr);
        eeprom.write_byte(*addr, byte_w2).unwrap();
        sleep(delay);
    }

    // read bytes and check

    for addr in addrs1.iter() {
        let byte_r = eeprom.read_byte(*addr).unwrap();
        println!("Read byte from address {:#x}: {:#x}", *addr, byte_r);
        assert_eq!(byte_w1, byte_r);
        sleep(delay);
    }

    for addr in addrs2.iter() {
        let byte_r = eeprom.read_byte(*addr).unwrap();
        println!("Read byte from address {:#x}: {:#x}", *addr, byte_r);
        assert_eq!(byte_w2, byte_r);
        sleep(delay);
    }
}
