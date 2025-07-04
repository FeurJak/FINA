use core::fmt;
use fina_common::io;

#[derive(Debug)]
pub enum SerializationError {
    NotEnoughSpace,
    InvalidData,
    UnexpectedFlags,
    IoError(io::Error),
}

impl core::error::Error for SerializationError {}

impl From<io::Error> for SerializationError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::NotEnoughSpace => {
                write!(f, "the last byte does not have enough space to encode the extra info bits")
            },
            Self::InvalidData => write!(f, "the input buffer contained invalid data"),
            Self::UnexpectedFlags => write!(f, "the call expects empty flags"),
            Self::IoError(err) => write!(f, "I/O error: {:?}", err),
        }
    }
}
