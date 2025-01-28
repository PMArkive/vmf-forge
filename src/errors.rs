//! This module defines the error types used in the VMF parser.

use pest::error::Error as PestError;
use std::io;

/// Represents an error that occurred during VMF parsing or serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmfError {
    /// An I/O error occurred.
    Io(io::ErrorKind),
    /// A parsing error occurred.
    Parse(Box<PestError<crate::parser::Rule>>),
    /// The VMF file has an invalid format.
    InvalidFormat(String),
    /// An error occurred while parsing an integer.
    ParseInt(std::num::ParseIntError, String),
    /// An error occurred while parsing a float.
    ParseFloat(std::num::ParseFloatError, String),
}

impl std::fmt::Display for VmfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VmfError::Io(err) => write!(f, "IO error: {}", err),
            VmfError::Parse(err) => write!(f, "Parse error: {}", err),
            VmfError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            VmfError::ParseInt(err, key) => {
                write!(f, "Integer parse error in key '{}': {}", key, err)
            }
            VmfError::ParseFloat(err, key) => {
                write!(f, "Float parse error in key '{}': {}", key, err)
            }
        }
    }
}

impl std::error::Error for VmfError {}

impl From<io::Error> for VmfError {
    fn from(err: io::Error) -> Self {
        VmfError::Io(err.kind())
    }
}

impl From<PestError<crate::parser::Rule>> for VmfError {
    fn from(err: PestError<crate::parser::Rule>) -> Self {
        VmfError::Parse(Box::new(err))
    }
}

impl From<std::num::ParseIntError> for VmfError {
    fn from(err: std::num::ParseIntError) -> Self {
        VmfError::ParseInt(err, "".to_string())
    }
}

impl From<std::num::ParseFloatError> for VmfError {
    fn from(err: std::num::ParseFloatError) -> Self {
        VmfError::ParseFloat(err, "".to_string())
    }
}

/// A type alias for `Result` that uses `VmfError` as the error type.
pub type VmfResult<T> = Result<T, VmfError>;
