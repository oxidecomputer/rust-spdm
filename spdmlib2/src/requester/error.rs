use crate::msgs::{Version, ReadError, WriteError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequesterError {
    Write(WriteError),
    Read(ReadError),

    // `got` is the code. TODO: Try to map this to a message name?
    UnexpectedMsg { expected: &'static str, got: u8 },

    //
    // Version related messages
    //
    NoSupportedVersions { received: Version },
}

impl From<WriteError> for RequesterError {
    fn from(e: WriteError) -> Self {
        RequesterError::Write(e)
    }
}

impl From<ReadError> for RequesterError {
    fn from(e: ReadError) -> Self {
        RequesterError::Read(e)
    }
}

