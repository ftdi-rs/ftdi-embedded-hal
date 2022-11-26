// https://github.com/ftdi-rs/ftdi-embedded-hal/issues/18
#[test]
#[ignore] // compile only
#[allow(dead_code)]
#[cfg_attr(not(feature = "libftd2xx"), allow(unused_imports))]
fn issue_18() {
    use ftdi_embedded_hal as hal;
    use std::sync::Arc;

    #[cfg(feature = "libftd2xx")]
    fn open() -> Result<hal::SpiDevice<libftd2xx::Ft4232h>, Box<dyn std::error::Error>> {
        let device = libftd2xx::Ft4232h::with_description("Dual RS232-HS A")?;
        let hal = Arc::new(hal::FtHal::init_freq(device, 3_000_000)?);
        let spi_dev = hal.spi_device(3)?;
        Ok(spi_dev)
    }
}
