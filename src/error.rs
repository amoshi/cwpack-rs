//! Sticky C return codes mapped to an idiomatic Rust error type.

use core::fmt;

/// CWPack `CWP_RC_*` codes (negative on error).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Error {
    Ok = 0,
    EndOfInput = -1,
    BufferOverflow = -2,
    BufferUnderflow = -3,
    MalformedInput = -4,
    WrongByteOrder = -5,
    ErrorInHandler = -6,
    IllegalCall = -7,
    MallocError = -8,
    Stopped = -9,
    TypeError = -10,
    ValueError = -11,
    WrongTimestampLength = -12,
}

impl Error {
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Ok,
            -1 => Self::EndOfInput,
            -2 => Self::BufferOverflow,
            -3 => Self::BufferUnderflow,
            -4 => Self::MalformedInput,
            -5 => Self::WrongByteOrder,
            -6 => Self::ErrorInHandler,
            -7 => Self::IllegalCall,
            -8 => Self::MallocError,
            -9 => Self::Stopped,
            -10 => Self::TypeError,
            -11 => Self::ValueError,
            -12 => Self::WrongTimestampLength,
            _ => Self::Stopped,
        }
    }

    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
