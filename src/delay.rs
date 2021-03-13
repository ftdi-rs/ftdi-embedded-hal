//! Implementation of the [`embedded_hal::blocking::delay`] traits.

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
    /// use ftd2xx_embedded_hal::Delay;
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

macro_rules! impl_delay_for {
    ($UXX:ty) => {
        impl embedded_hal::blocking::delay::DelayMs<$UXX> for Delay {
            fn delay_ms(&mut self, ms: $UXX) {
                std::thread::sleep(std::time::Duration::from_millis(ms.into()))
            }
        }

        impl embedded_hal::blocking::delay::DelayUs<$UXX> for Delay {
            fn delay_us(&mut self, us: $UXX) {
                std::thread::sleep(std::time::Duration::from_micros(us.into()))
            }
        }
    };
}

impl_delay_for!(u8);
impl_delay_for!(u16);
impl_delay_for!(u32);
impl_delay_for!(u64);
