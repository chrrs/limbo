use std::{io::Cursor, string::FromUtf8Error};

use thiserror::Error;

pub use variable::*;

macro_rules! packet_enum {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $super:ident {
            $($variant:ident = $id:expr),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }

        impl crate::protocol::Readable for $name {
            fn read_from(buffer: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::protocol::ProtocolError> {
                let value = $super::read_from(buffer)?;
                match value.into() {
                    $($id => Ok(Self::$variant),)*
                    _ => Err(crate::protocol::ProtocolError::InvalidEnumVariant)
                }
            }
        }
    };
}

pub mod info;
mod io;
pub mod packets;
mod variable;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("variable-length number too large")]
    VariableTooLarge,

    #[error("string contains non-UTF8 characters")]
    InvalidString(#[from] FromUtf8Error),

    #[error("packet with id={0} does not exist")]
    InvalidPacketId(i32),

    #[error("enum variant with id does not exist")]
    InvalidEnumVariant,

    #[error("failed to read from buffer")]
    Io(#[from] std::io::Error),
}

pub trait Readable: Sized {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError>;
}

pub trait Writable {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError>;
}
