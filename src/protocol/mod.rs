use std::{
    io::{Cursor, Write},
    string::FromUtf8Error,
};

use thiserror::Error;

pub use variable::*;

macro_rules! packet {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $id:literal = $packet:ident {
                    $($field:ident: $typ:ident),*
                    $(,)?
                }
            ),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $packet {
                    $($field: $typ),*
                }
            ),*
        }

        impl crate::protocol::Readable for $name {
            fn read_from(buffer: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::protocol::ProtocolError> {
                match crate::protocol::VarInt::read_from(buffer)?.0 {
                    $(
                        $id => Ok(Self::$packet {
                            $(
                                $field: $typ::read_from(buffer)?,
                            )*
                        }),
                    )*
                    id => Err(crate::protocol::ProtocolError::InvalidPacketId(id)),
                }
            }
        }

        impl crate::protocol::Writable for $name {
            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::protocol::ProtocolError> {
                match self {
                    $(
                        Self::$packet { $($field),* } => {
                            crate::protocol::VarInt($id).write_to(buffer)?;
                            $($field.write_to(buffer)?;)*
                            Ok(())
                        },
                    )*
                }
            }
        }
    };
}

macro_rules! packet_enum {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $super:ident {
            $($variant:ident = $id:ident$(($arg:expr))?),*
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
                match value {
                    $($id$(($arg))? => Ok(Self::$variant),)*
                    _ => Err(crate::protocol::ProtocolError::InvalidEnumVariant)
                }
            }
        }

        impl crate::protocol::Writable for $name {
            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::protocol::ProtocolError> {
                match self {
                    $(Self::$variant => Ok($id$(($arg))?.write_to(buffer)?),)*
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
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError>;
}
