use crate::error::Error;
use crate::error::ErrorKind::I2cNoAck;
use crate::{FtInner, PinUse};
use ftdi_mpsse::{ClockBitsIn, ClockBitsOut, MpsseCmdBuilder, MpsseCmdExecutor};
use std::sync::{Arc, Mutex};

/// SCL bitmask
const SCL: u8 = 1 << 0;
/// SDA bitmask
const SDA: u8 = 1 << 1;

const BITS_IN: ClockBitsIn = ClockBitsIn::MsbPos;
const BITS_OUT: ClockBitsOut = ClockBitsOut::MsbNeg;

/// FTDI I2C interface.
///
/// This is created by calling [`FtHal::i2c`].
///
/// [`FtHal::i2c`]: crate::FtHal::i2c
#[derive(Debug)]
pub struct I2c<Device: MpsseCmdExecutor> {
    /// Parent FTDI device.
    mtx: Arc<Mutex<FtInner<Device>>>,
    /// Length of the start, repeated start, and stop conditions.
    ///
    /// The units for these are dimensionless number of MPSSE commands.
    /// More MPSSE commands roughly correlates to more time.
    start_stop_cmds: u8,
    /// Send I2C commands faster.
    fast: bool,
}

impl<Device, E> I2c<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    pub(crate) fn new(mtx: Arc<Mutex<FtInner<Device>>>) -> Result<I2c<Device>, Error<E>> {
        {
            let mut lock = mtx.lock().expect("Failed to aquire FTDI mutex");

            lock.allocate_pin(0, PinUse::I2c);
            lock.allocate_pin(1, PinUse::I2c);
            lock.allocate_pin(2, PinUse::I2c);

            // clear direction and value of first 3 pins

            lock.direction &= !0x07;
            lock.value &= !0x07;
            // AD0: SCL
            // AD1: SDA (master out)
            // AD2: SDA (master in)
            // pins are set as input (tri-stated) in idle mode

            // set GPIO pins to new state
            let cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
                .set_gpio_lower(lock.value, lock.direction)
                .enable_3phase_data_clocking()
                .send_immediate();
            lock.ft.send(cmd.as_slice())?;
        }

        Ok(I2c {
            mtx,
            start_stop_cmds: 3,
            fast: false,
        })
    }

    /// Set the length of start and stop conditions.
    ///
    /// This is an advanced feature that most people will not need to touch.
    /// I2C start and stop conditions are generated with a number of MPSSE
    /// commands.  This sets the number of MPSSE command generated for each
    /// stop and start condition.  An increase in the number of MPSSE commands
    /// roughtly correlates to an increase in the duration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "libftd2xx")]
    /// # {
    /// let device = libftd2xx::Ft2232h::with_description("Dual RS232-HS A")?;
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// let mut i2c = hal.i2c()?;
    /// i2c.set_stop_start_len(10);
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_stop_start_len(&mut self, start_stop_cmds: u8) {
        self.start_stop_cmds = start_stop_cmds
    }

    /// Enable faster I2C transactions by sending commands in a single write.
    ///
    /// This is disabled by default.
    ///
    /// Normally the I2C methods will send commands with a delay after each
    /// slave ACK to read from the USB device.
    /// Enabling this will send I2C commands without a delay, but slave ACKs
    /// will only be checked at the end of each call to `read`, `write`, or
    /// `write_read`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// # #[cfg(feature = "ftdi")]
    /// # {
    /// let device = ftdi::find_by_vid_pid(0x0403, 0x6014)
    ///     .interface(ftdi::Interface::A)
    ///     .open()?;
    ///
    /// let hal = hal::FtHal::init_freq(device, 3_000_000)?;
    /// let mut i2c = hal.i2c()?;
    /// i2c.set_fast(true);
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_fast(&mut self, fast: bool) {
        self.fast = fast
    }

    fn read_fast(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error<E>> {
        assert!(!buffer.is_empty(), "buffer must be a non-empty slice");

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        // ST
        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD+R
            .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
            .clock_bits_out(BITS_OUT, (address << 1) | 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1);

        for idx in 0..buffer.len() {
            // Bn
            mpsse_cmd = mpsse_cmd
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 8);
            if idx == buffer.len() - 1 {
                // NMAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x80, 1)
            } else {
                // MAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x00, 1)
            }
        }

        // SP
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // Idle
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: [u8; 1] = [0; 1];
        lock.ft.recv(&mut ack_buf)?;
        lock.ft.recv(buffer)?;

        if (ack_buf[0] & 0b1) != 0x00 {
            return Err(Error::Hal(I2cNoAck));
        }

        Ok(())
    }

    fn read_slow(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error<E>> {
        assert!(!buffer.is_empty(), "buffer must be a non-empty slice");

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        // ST
        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD+R
            .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
            .clock_bits_out(BITS_OUT, (address << 1) | 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: [u8; 1] = [0; 1];
        lock.ft.recv(&mut ack_buf)?;
        if (ack_buf[0] & 0b1) != 0x00 {
            return Err(Error::Hal(I2cNoAck));
        }

        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for idx in 0..buffer.len() {
            // Bn
            mpsse_cmd = mpsse_cmd
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 8);
            if idx == buffer.len() - 1 {
                // NMAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x80, 1)
            } else {
                // MAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x00, 1)
            }
        }

        // SP
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // Idle
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        lock.ft.recv(buffer)?;

        Ok(())
    }

    fn write_fast(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error<E>> {
        assert!(!bytes.is_empty(), "bytes must be a non-empty slice");

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();

        // ST
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD+W
            .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
            .clock_bits_out(BITS_OUT, addr << 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1);

        for byte in bytes.iter() {
            mpsse_cmd = mpsse_cmd
                // Bi
                .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                .clock_bits_out(BITS_OUT, *byte, 8)
                // SAK
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 1);
        }

        // SP
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // Idle
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: Vec<u8> = vec![0; 1 + bytes.len()];
        lock.ft.recv(ack_buf.as_mut_slice())?;
        if ack_buf.iter().any(|&ack| (ack & 0b1) != 0x00) {
            Err(Error::Hal(I2cNoAck))
        } else {
            Ok(())
        }
    }

    fn write_slow(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error<E>> {
        assert!(!bytes.is_empty(), "bytes must be a non-empty slice");

        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        // ST
        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(SCL | SDA | lock.value, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(SCL | lock.value, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD+W
            .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
            .clock_bits_out(BITS_OUT, addr << 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: [u8; 1] = [0; 1];
        lock.ft.recv(&mut ack_buf)?;
        if (ack_buf[0] & 0b1) != 0x00 {
            return Err(Error::Hal(I2cNoAck));
        }

        for (idx, byte) in bytes.iter().enumerate() {
            let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
                // Bi
                .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                .clock_bits_out(BITS_OUT, *byte, 8)
                // SAK
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 1);

            // last byte
            if idx == bytes.len() - 1 {
                // SP
                for _ in 0..self.start_stop_cmds {
                    mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                }
                for _ in 0..self.start_stop_cmds {
                    mpsse_cmd =
                        mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
                }
                for _ in 0..self.start_stop_cmds {
                    mpsse_cmd =
                        mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
                }

                // Idle
                mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, lock.direction)
            }

            mpsse_cmd = mpsse_cmd.send_immediate();

            lock.ft.send(mpsse_cmd.as_slice())?;
            let mut ack_buf: [u8; 1] = [0; 1];
            lock.ft.recv(&mut ack_buf)?;
            if (ack_buf[0] & 0b1) != 0x00 {
                return Err(Error::Hal(I2cNoAck));
            }
        }

        Ok(())
    }

    fn write_read_fast(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Error<E>> {
        assert!(!bytes.is_empty(), "bytes must be a non-empty slice");
        assert!(!buffer.is_empty(), "buffer must be a non-empty slice");

        // lock at the start to prevent GPIO from being modified while we build
        // the MPSSE command
        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();

        // ST
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD + W
            .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
            .clock_bits_out(BITS_OUT, address << 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1);

        for byte in bytes {
            mpsse_cmd = mpsse_cmd
                // Oi
                .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                .clock_bits_out(BITS_OUT, *byte, 8)
                // SAK
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 1);
        }

        // SR
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD + R
            .clock_bits_out(BITS_OUT, (address << 1) | 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1);

        for idx in 0..buffer.len() {
            mpsse_cmd = mpsse_cmd
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 8);
            if idx == buffer.len() - 1 {
                // NMAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x80, 1)
            } else {
                // MAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x00, 1)
            }
        }

        // SP
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // Idle
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: Vec<u8> = vec![0; 2 + bytes.len()];
        lock.ft.recv(&mut ack_buf)?;
        lock.ft.recv(buffer)?;

        if ack_buf.iter().any(|&ack| (ack & 0b1) != 0x00) {
            Err(Error::Hal(I2cNoAck))
        } else {
            Ok(())
        }
    }

    fn write_read_slow(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Error<E>> {
        assert!(!bytes.is_empty(), "bytes must be a non-empty slice");
        assert!(!buffer.is_empty(), "buffer must be a non-empty slice");

        // lock at the start to prevent GPIO from being modified while we build
        // the MPSSE command
        let mut lock = self.mtx.lock().expect("Failed to aquire FTDI mutex");

        // ST
        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD + W
            .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
            .clock_bits_out(BITS_OUT, address << 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: [u8; 1] = [0; 1];
        lock.ft.recv(&mut ack_buf)?;
        if (ack_buf[0] & 0b1) != 0x00 {
            return Err(Error::Hal(I2cNoAck));
        }

        for byte in bytes {
            let mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new()
                // Oi
                .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                .clock_bits_out(BITS_OUT, *byte, 8)
                // SAK
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 1)
                .send_immediate();

            lock.ft.send(mpsse_cmd.as_slice())?;
            let mut ack_buf: [u8; 1] = [0; 1];
            lock.ft.recv(&mut ack_buf)?;
            if (ack_buf[0] & 0b1) != 0x00 {
                return Err(Error::Hal(I2cNoAck));
            }
        }

        // SR
        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // SAD + R
            .clock_bits_out(BITS_OUT, (address << 1) | 1, 8)
            // SAK
            .set_gpio_lower(lock.value, SCL | lock.direction)
            .clock_bits_in(BITS_IN, 1)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        let mut ack_buf: [u8; 1] = [0; 1];
        lock.ft.recv(&mut ack_buf)?;
        if (ack_buf[0] & 0b1) != 0x00 {
            return Err(Error::Hal(I2cNoAck));
        }

        let mut mpsse_cmd: MpsseCmdBuilder = MpsseCmdBuilder::new();
        for idx in 0..buffer.len() {
            mpsse_cmd = mpsse_cmd
                .set_gpio_lower(lock.value, SCL | lock.direction)
                .clock_bits_in(BITS_IN, 8);
            if idx == buffer.len() - 1 {
                // NMAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x80, 1)
            } else {
                // MAK
                mpsse_cmd = mpsse_cmd
                    .set_gpio_lower(lock.value, SCL | SDA | lock.direction)
                    .clock_bits_out(BITS_OUT, 0x00, 1)
            }
        }

        // SP
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL, SCL | SDA | lock.direction)
        }
        for _ in 0..self.start_stop_cmds {
            mpsse_cmd = mpsse_cmd.set_gpio_lower(lock.value | SCL | SDA, SCL | SDA | lock.direction)
        }

        mpsse_cmd = mpsse_cmd
            // Idle
            .set_gpio_lower(lock.value, lock.direction)
            .send_immediate();

        lock.ft.send(mpsse_cmd.as_slice())?;
        lock.ft.recv(buffer)?;

        Ok(())
    }
}

impl<Device, E> eh0::blocking::i2c::Read for I2c<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error<E>> {
        if self.fast {
            self.read_fast(address, buffer)
        } else {
            self.read_slow(address, buffer)
        }
    }
}

impl<Device, E> eh0::blocking::i2c::Write for I2c<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error<E>> {
        if self.fast {
            self.write_fast(addr, bytes)
        } else {
            self.write_slow(addr, bytes)
        }
    }
}

impl<Device, E> eh0::blocking::i2c::WriteRead for I2c<Device>
where
    Device: MpsseCmdExecutor<Error = E>,
    E: std::error::Error,
    Error<E>: From<E>,
{
    type Error = Error<E>;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error<E>> {
        if self.fast {
            self.write_read_fast(address, bytes, buffer)
        } else {
            self.write_read_slow(address, bytes, buffer)
        }
    }
}
