use eh1::i2c::I2c;
use ftdi_embedded_hal as hal;

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ftdi")] {
            let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
                .interface(ftdi::Interface::A)
                .open()
                .unwrap();
        } else if #[cfg(feature = "libftd2xx")] {
            let device: libftd2xx::Ft232h = libftd2xx::Ftdi::new().unwrap().try_into().unwrap();
        } else {
            compile_error!("Enable feature 'ftdi' or 'libftd2xx'");
        }
    }

    let hal = hal::FtHal::init_default(device).unwrap();
    let mut i2c = hal.i2c().unwrap();

    println!("     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f");
    for row in 0..8 {
        print!("{:02x}: ", row << 4);
        for col in 0..16 {
            let addr = (row << 4) | col;

            // For addresses i2cdetect typically skips:
            // 0x00..=0x07 and 0x78..=0x7F
            if addr < 0x08 || addr > 0x77 {
                print!("   ");
                continue;
            }

            let mut buf = [0u8; 1];
            if I2c::write_read(&mut i2c, addr, &[], &mut buf).is_ok() {
                print!("{:02x} ", addr);
            } else {
                print!("-- ");
            }
        }
        println!();
    }
}
