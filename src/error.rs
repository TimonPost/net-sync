//! Module that provides code for error handling.

use std::fmt::{Display, Formatter};
use std::io;
use std::io::Error;

/// Wrapper for all errors that can occur in `net-sync`.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error has occurred related to data compression.
    CompressionError(String),
    // An error has occurred related to data serialization.
    SerializationError(String),
    IoError(io::Error),
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::IoError(e) => write!(fmt, "IO error occurred: {:?}", e),
            ErrorKind::CompressionError(e) => write!(fmt, "Serialisation error occurred: {:?}", e),
            ErrorKind::SerializationError(e) => {
                write!(fmt, "Serialization error occurred: {:?}", e)
            }
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(error: io::Error) -> Self {
        ErrorKind::IoError(error)
    }
}
