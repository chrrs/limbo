use std::io::Cursor;

use thiserror::Error;

pub use variable::*;

mod variable;

#[derive(Error, Debug)]
pub enum Error {
    #[error("variable-length number too large")]
    VariableTooLarge,

    #[error("failed to read from buffer")]
    IoError(#[from] std::io::Error),
}

pub trait Readable: Sized {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, Error>;
}

pub trait Writable {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), Error>;
}
