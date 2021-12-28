use std::{io::Cursor, string::FromUtf8Error};

use thiserror::Error;

pub use variable::*;

mod io;
pub mod packets;
mod variable;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("variable-length number too large")]
    VariableTooLarge,

    #[error("string contains non-UTF8 characters")]
    InvalidString(#[from] FromUtf8Error),

    #[error("failed to read from buffer")]
    Io(#[from] std::io::Error),
}

pub trait Readable: Sized {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError>;
}

pub trait Writable {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError>;
}
