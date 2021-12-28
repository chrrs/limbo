use std::io::Cursor;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
}

pub trait Readable: Sized {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, Error>;
}

pub trait Writable {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), Error>;
}
