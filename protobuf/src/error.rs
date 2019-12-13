use std::error::Error;
use std::fmt;
use std::io;
use std::str;

use crate::wire_format::WireType;

/// `Result` alias for `ProtobufError`
pub type ProtobufResult<T> = Result<T, ProtobufError>;

/// Enum values added here for diagnostic purposes.
/// Users should not depend on specific values.
#[derive(Debug)]
pub enum WireError {
    UnexpectedEof,
    UnexpectedWireType(WireType),
    IncorrectTag(u32),
    // unused since https://github.com/stepancheg/rust-protobuf/issues/318
    IncompleteMap,
    IncorrectVarint,
    Utf8Error,
    InvalidEnumValue(i32),
    OverRecursionLimit,
    TruncatedMessage,
    Other,
}

/// Generic protobuf error
#[derive(Debug)]
pub enum ProtobufError {
    /// I/O error when reading or writing
    IoError(io::Error),
    /// Malformed input
    WireError(WireError),
    /// Protocol contains a string which is not valid UTF-8 string
    Utf8(str::Utf8Error),
    /// Not all required fields of message set.
    MessageNotInitialized(&'static str),
}

impl fmt::Display for ProtobufError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for ProtobufError {
    fn description(&self) -> &str {
        match self {
            // not sure that cause should be included in message
            &ProtobufError::IoError(ref e) => e.description(),
            &ProtobufError::WireError(ref e) => match *e {
                WireError::Utf8Error => "invalid UTF-8 sequence",
                WireError::UnexpectedWireType(..) => "unexpected wire type",
                WireError::InvalidEnumValue(..) => "invalid enum value",
                WireError::IncorrectTag(..) => "incorrect tag",
                WireError::IncorrectVarint => "incorrect varint",
                WireError::IncompleteMap => "incomplete map",
                WireError::UnexpectedEof => "unexpected EOF",
                WireError::OverRecursionLimit => "over recursion limit",
                WireError::TruncatedMessage => "truncated message",
                WireError::Other => "other error",
            },
            &ProtobufError::Utf8(ref e) => &e.description(),
            &ProtobufError::MessageNotInitialized { .. } => "not all message fields set",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            &ProtobufError::IoError(ref e) => Some(e),
            &ProtobufError::Utf8(ref e) => Some(e),
            &ProtobufError::WireError(..) => None,
            &ProtobufError::MessageNotInitialized { .. } => None,
        }
    }
}

impl From<io::Error> for ProtobufError {
    fn from(err: io::Error) -> Self {
        ProtobufError::IoError(err)
    }
}

impl From<str::Utf8Error> for ProtobufError {
    fn from(err: str::Utf8Error) -> Self {
        ProtobufError::Utf8(err)
    }
}

impl From<ProtobufError> for io::Error {
    fn from(err: ProtobufError) -> Self {
        match err {
            ProtobufError::IoError(e) => e,
            ProtobufError::WireError(e) => {
                io::Error::new(io::ErrorKind::InvalidData, ProtobufError::WireError(e))
            }
            ProtobufError::MessageNotInitialized(message) => io::Error::new(
                io::ErrorKind::InvalidInput,
                ProtobufError::MessageNotInitialized(message),
            ),
            e => io::Error::new(io::ErrorKind::Other, Box::new(e)),
        }
    }
}
