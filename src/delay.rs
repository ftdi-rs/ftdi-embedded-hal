//! Implementation of the [`eh0::blocking::delay`] and [`eh1::delay`]
//! traits.

/// Delay structure.
///
/// This is an empty structure that forwards delays to [`std::thread::sleep`].
///
/// [`sleep`]: std::thread::sleep
#[derive(Debug, Clone, Copy)]
pub struct Delay {
    _0: (),
}

impl Delay {
    /// Create a new delay structure.
    ///
    /// # Example
    ///
    /// ```
    /// use ftdi_embedded_hal::Delay;
    ///
    /// let mut my_delay: Delay = Delay::new();
    /// ```
    pub const fn new() -> Delay {
        Delay { _0: () }
    }
}

impl Default for Delay {
    fn default() -> Self {
        Delay::new()
    }
}

impl eh1::delay::DelayUs for Delay {
    fn delay_us(&mut self, us: u32) {
        std::thread::sleep(std::time::Duration::from_micros(us.into()))
    }

    fn delay_ms(&mut self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms.into()))
    }
}

macro_rules! impl_eh0_delay_for {
    ($UXX:ty) => {
        impl eh0::blocking::delay::DelayMs<$UXX> for Delay {
            fn delay_ms(&mut self, ms: $UXX) {
                std::thread::sleep(std::time::Duration::from_millis(ms.into()))
            }
        }

        impl eh0::blocking::delay::DelayUs<$UXX> for Delay {
            fn delay_us(&mut self, us: $UXX) {
                std::thread::sleep(std::time::Duration::from_micros(us.into()))
            }
        }
    };
}

impl_eh0_delay_for!(u8);
impl_eh0_delay_for!(u16);
impl_eh0_delay_for!(u32);
impl_eh0_delay_for!(u64);
