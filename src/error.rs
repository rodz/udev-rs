use std::ffi::CStr;
use std::fmt;
use std::io;
use std::str;

use std::error::Error as StdError;
use std::result::Result as StdResult;

use libc::c_int;

/// A `Result` type for libudev operations.
pub type Result<T> = StdResult<T, Error>;

/// Types of errors that occur in libudev.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Allocation failed
    NoMem,
    /// Invalid arguments
    InvalidInput,
    /// I/O Error
    Io(io::ErrorKind),
}

/// The error type for libudev operations.
#[derive(Debug)]
pub struct Error {
    errno: c_int,
}

impl Error {
    fn strerror(&self) -> &str {
        unsafe { str::from_utf8_unchecked(CStr::from_ptr(libc::strerror(self.errno)).to_bytes()) }
    }

    /// Returns the corresponding `Kind` for this error.
    pub fn kind(&self) -> Kind {
        match self.errno {
            libc::ENOMEM => Kind::NoMem,
            libc::EINVAL => Kind::InvalidInput,
            errno => Kind::Io(io::Error::from_raw_os_error(errno).kind()),
        }
    }

    /// Returns a description of the error.
    pub fn description(&self) -> &str {
        self.strerror()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        fmt.write_str(self.strerror())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.strerror()
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> Self {
        let io_error_kind = match error.kind() {
            Kind::Io(kind) => kind,
            Kind::InvalidInput => io::ErrorKind::InvalidInput,
            Kind::NoMem => io::ErrorKind::Other,
        };

        Self::new(io_error_kind, error.strerror())
    }
}

pub fn from_errno(errno: c_int) -> Error {
    Error { errno: -errno }
}
