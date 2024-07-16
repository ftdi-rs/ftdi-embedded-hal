use std::io::Read;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::error::Error;
use crate::gpiosbb::{InputPinSbb, OutputPinSbb, Pin};
use crate::{GpioByte, PinUse};

use ftdi;

/// FTHal Synchronous Bit-Bang mode settings struct.
/// The defaults are a sensible starting point:
///
/// * Reset the FTDI device.
/// * 4k USB transfer size for read and write.
/// * 16ms latency timer.
/// * 100kHz clock frequency.
#[derive(Debug)]
pub struct FtHalSbbSettings {
    reset: bool,
    read_chunksize: u32,
    write_chunksize: u32,
    latency_timer_ms: u8,
    clock_frequency: u32,
}

impl Default for FtHalSbbSettings {
    fn default() -> Self {
        FtHalSbbSettings {
            reset: true,
            read_chunksize: 4096,
            write_chunksize: 4096,
            latency_timer_ms: 16,
            clock_frequency: 100_000,
        }
    }
}

// Internal struct to hold in the mutex.
// Need the FTDI device, but also the pin directions and types.
pub(crate) struct FtInnerSbb {
    pub(crate) ft: ftdi::Device,
    pub(crate) lower: GpioByte,
    pub(crate) upper: GpioByte,
}

impl FtInnerSbb {
    /// Allocate a pin in the lower byte for a specific use.
    pub fn allocate_pin(&mut self, idx: u8, purpose: PinUse) {
        assert!(idx < 8, "Pin index {idx} is out of range 0 - 7");

        if let Some(current) = self.lower.pins[usize::from(idx)] {
            panic!(
            "Unable to allocate pin {idx} for {purpose}, pin is already allocated for {current}"
            );
        } else {
            self.lower.pins[usize::from(idx)] = Some(purpose)
        }
    }

    /// Allocate a pin for a specific use.
    pub fn allocate_pin_any(&mut self, pin: Pin, purpose: PinUse) {
        let (byte, idx) = match pin {
            Pin::Lower(idx) => (&mut self.lower, idx),
            Pin::Upper(idx) => (&mut self.upper, idx),
        };
        assert!(idx < 8, "Pin index {idx} is out of range 0 - 7");

        if let Some(current) = byte.pins[usize::from(idx)] {
            panic!(
            "Unable to allocate pin {idx} for {purpose}, pin is already allocated for {current}"
            );
        } else {
            byte.pins[usize::from(idx)] = Some(purpose)
        }
    }
}

/// For the FT4232H, ports C and D do not support the MPSSE. Only UART and
/// bit-bang modes are possible. This means that a different method of port
/// access is required. As there is no MPSSE, only GPIO mode is supported.
///
/// The GPIO operations are implemented using the synchronous bit-bang mode.
/// This mode keeps stimulus-response in lock-step, which is the expected
/// behavior when setting and getting GPIO pin states. There is a gotcha that
/// is explained in the FT4232H data sheet, V2.6, Ch. 4.5.2, p.23:
///
/// With Synchronous Bit-Bang mode, data will only be sent-out by the chip
/// if there is space in the chip's USB TX FIFO for data to be read from the
/// parallel interface pins. The data bus parallel I/O pins are read first,
/// before data from the USB RX FIFO is transmitted. It is therefore 1 byte
/// behind the output, and so to read the inputs for the byte that you have
/// just sent, another byte must be sent.
///
/// For example:
/// (1) Pins start at 0xFF
/// - Send 0x55,0xAA
/// - Pins go to 0x55 and then to 0xAA
/// - Data read = 0xFF,0x55
///
/// (2) Pins start at 0xFF
/// - Send 0x55,0xAA,0xAA
///   (repeat the last byte sent)
/// - Pins go to 0x55 and then to 0xAA
/// - Data read = 0xFF,0x55,0xAA
///
/// In the code below, the (2) sequence is used.
///
/// Because a write is required to precede a read, the (at least) doubling of
/// the last written data byte is implemented in the gpio read (get): the
/// complete gpio output byte set-up for the previous gpio write (set) is
/// repeated, so that any read is always preceded by (at least) a double write
/// of the last gpio output byte.
///
/// To avoid potential problems of the chip's USB TX FIFO overflowing after a
/// long sequence of writes (set), all data is read from it, both in the set
/// and get functions. Writes cannot occur when this FIFO is full, so
/// explicitly clearing it out before any write is not a bad idea.
pub struct FtHalSbb<E> {
    mtx: Arc<Mutex<FtInnerSbb>>,
    // To satisfy the compiler. We need a type parameter <E> on the struct,
    // otherwise the impl constraints fail. But it is not used in the struct.
    // The use of a PhantomData member that uses <E> solves this problem.
    _p: PhantomData<E>,
}

impl<E> FtHalSbb<E>
where
    E: std::error::Error,
    Error<E>: From<ftdi::Error>,
{
    /// Initialize the FTDI synchronous bit-bang interface with custom values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ftdi_embedded_hal as hal;
    ///
    /// let sbb = FtHalSbbSettings {
    ///     reset: false,
    ///     read_chunksize: 4096,
    ///     write_chunksize: 4096,
    ///     latency_timer: 32,
    ///     clock_frequency: 1_000_000,
    /// };
    ///
    /// # #[cfg(feature = "ftdi")]
    /// # {
    /// let device = ftdi::find_by_vid_pid(0x0403, 0x6011)
    /// .interface(ftdi::Interface::D)
    /// .open()
    /// .unwrap();
    ///
    /// let hal_cfg = FtHalSbbSettings::default();
    /// let hal = FtHalSbb::init(epe_if_d, hal_cfg).unwrap();
    /// # }
    /// # Ok::<(), std::boxed::Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`FtHalSbbSettings`]: crate::fthalsbb::FtHalSbbSettings
    pub fn init(device: ftdi::Device, settings: FtHalSbbSettings) -> Result<FtHalSbb<E>, Error<E>> {
        // Keep the device handler and pin settings together in a struct.
        // The arc mutex will eventually wrap this in turn.
        let mut inner = FtInnerSbb {
            ft: device, // Holds the ftdi::Device struct.
            lower: GpioByte {
                direction: 0,    // Initialize all pins as inputs. (Safer!)
                value: 0,        // All to zeros.
                pins: [None; 8], // No pins uses allocated yet.
            },
            upper: GpioByte {
                direction: 0,    // Initialize all pins as inputs. (Safer!)
                value: 0,        // All to zeros.
                pins: [None; 8], // No pins uses allocated yet.
            },
        };

        // Initialize the ftdi device using the passed configuration struct.
        // The data is clocked-out at a rate controlled by the baud rate
        // generator. See: FT4232H data sheet, V2.6, Ch. 4.5.1, p. 23.
        if settings.reset {
            inner.ft.usb_reset()?;
        }
        inner.ft.usb_purge_buffers()?;
        inner.ft.set_read_chunksize(settings.read_chunksize);
        inner.ft.set_write_chunksize(settings.write_chunksize);
        inner.ft.set_latency_timer(settings.latency_timer_ms)?;
        inner.ft.set_baud_rate(settings.clock_frequency)?;

        // Configure synchronous bit-bang mode and the pin direction set above.
        // When pins are assigned later, the direction is modified accordingly.
        inner
            .ft
            .set_bitmode(inner.lower.direction, ftdi::BitMode::SyncBB)?;

        // Purge the read buffer and discard.
        // There can be a few stray bytes in it at this point. Also, a write
        // will only execute if there is space in the chip's USB TX FIFO, so
        // it is best to insure it is empty from the get go.
        let mut stray_bytes = vec![];
        inner.ft.read_to_end(&mut stray_bytes)?;

        Ok(FtHalSbb {
            mtx: Arc::new(Mutex::new(inner)),
            _p: PhantomData,
        })
    }

    /// Acquire the digital output pin 0 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad0(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(0))
    }

    /// Acquire the digital input pin 0 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi0(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(0))
    }

    /// Acquire the digital output pin 1 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad1(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(1))
    }

    /// Acquire the digital input pin 1 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi1(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(1))
    }

    /// Acquire the digital output pin 2 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad2(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(2))
    }

    /// Acquire the digital input pin 2 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi2(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(2))
    }

    /// Acquire the digital output pin 3 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad3(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(3))
    }

    /// Acquire the digital input pin 3 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi3(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(3))
    }

    /// Acquire the digital output pin 4 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad4(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(4))
    }

    /// Acquire the digital input pin 4 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi4(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(4))
    }

    /// Acquire the digital output pin 5 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad5(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(5))
    }

    /// Acquire the digital input pin 5 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi5(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(5))
    }

    /// Acquire the digital output pin 6 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad6(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(6))
    }

    /// Acquire the digital input pin 6 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi6(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(6))
    }

    /// Acquire the digital output pin 7 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn ad7(&self) -> Result<OutputPinSbb<E>, Error<E>> {
        OutputPinSbb::new(self.mtx.clone(), Pin::Lower(7))
    }

    /// Acquire the digital input pin 7 for the FTx232H, using synchronous
    /// bit-bang mode.
    ///
    /// # Panics
    ///
    /// Panics if the pin is already in-use.
    pub fn adi7(&self) -> Result<InputPinSbb<E>, Error<E>> {
        InputPinSbb::new(self.mtx.clone(), Pin::Lower(7))
    }
}
