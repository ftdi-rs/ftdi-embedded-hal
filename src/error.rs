use std::fmt;
use std::io;

/// Error type.
#[derive(Debug)]
pub enum Error<E: std::error::Error> {
    /// ftdi-embedded-hal implementation specific error.
    Hal(ErrorKind),
    /// IO error.
    Io(io::Error),
    /// Backend specific error.
    Backend(E),
}

/// Internal HAL errors
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// No ACK from the I2C slave
    I2cNoAck,
}

impl ErrorKind {
    fn as_str(&self) -> &str {
        match *self {
            ErrorKind::I2cNoAck => "No ACK from slave",
        }
    }
}

impl<E: std::error::Error> eh1::digital::Error for Error<E> {
    fn kind(&self) -> eh1::digital::ErrorKind {
        eh1::digital::ErrorKind::Other
    }
}

impl<E: std::error::Error> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => e.fmt(f),
            Error::Backend(e) => fmt::Display::fmt(&e, f),
            Error::Hal(e) => write!(f, "A regular error occurred {:?}", e.as_str()),
        }
    }
}

impl<E: std::error::Error> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => e.source(),
            Error::Backend(e) => e.source(),
            Error::Hal(_) => None,
        }
    }
}

impl<E: std::error::Error> From<io::Error> for Error<E> {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(feature = "ftdi")]
impl From<ftdi::Error> for Error<ftdi::Error> {
    fn from(e: ftdi::Error) -> Self {
        Error::Backend(e)
    }
}

#[cfg(feature = "libftd2xx")]
impl From<libftd2xx::TimeoutError> for Error<libftd2xx::TimeoutError> {
    fn from(e: libftd2xx::TimeoutError) -> Self {
        Error::Backend(e)
    }
}
