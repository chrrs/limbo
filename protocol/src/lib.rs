use std::{
    io::{Cursor, Read, Write},
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
                    $($field:ident: $typ:ident$(<$generics:ident>)?),*
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
                    $($field: $typ$(<$generics>)?),*
                }
            ),*
        }

        impl crate::Packet for $name {
            fn read_from(buffer: &mut dyn std::io::Read) -> Result<Self, crate::ReadError> {
                use crate::PacketField;

                match crate::VarInt::read_from(buffer).map_err(crate::ReadError::ReadPacketId)?.0 {
                    $(
                        $id => Ok(Self::$packet {
                            $(
                                $field: $typ::read_from(buffer)
                                    .map_err(|e| crate::ReadError::Field(stringify!($field), e))?,
                            )*
                        }),
                    )*
                    id => Err(crate::ReadError::UnrecognizedPacketId(id as usize)),
                }
            }

            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::WriteError> {
                use crate::PacketField;

                match self {
                    $(
                        Self::$packet { $($field),* } => {
                            crate::VarInt($id).write_to(buffer)
                                .map_err(crate::WriteError::WritePacketId)?;

                            $(
                                $field.write_to(buffer)
                                    .map_err(|e| crate::WriteError::Field(stringify!($field), e))?;
                            )*

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
            $($variant:ident = $class:ident ( $arg:expr )),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }

        impl crate::PacketField for $name {
            fn read_from(buffer: &mut dyn std::io::Read) -> Result<Self, crate::FieldReadError> {
                let value = $super::read_from(buffer)?;
                match value {
                    $($class($arg) => Ok(Self::$variant),)*
                    id => Err(crate::FieldReadError::InvalidEnumId(format!("{}", id))),
                }
            }

            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::FieldWriteError> {
                match self {
                    $(Self::$variant => Ok($class($arg).write_to(buffer)?),)*
                }
            }
        }
    };

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

        impl crate::PacketField for $name {
            fn read_from(buffer: &mut dyn std::io::Read) -> Result<Self, crate::FieldReadError> {
                let value = $super::read_from(buffer)?;
                match value {
                    $($id => Ok(Self::$variant),)*
                    id => Err(crate::FieldReadError::InvalidEnumId(format!("{}", id))),
                }
            }

            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::FieldWriteError> {
                match self {
                    $(Self::$variant => Ok(($id as $super).write_to(buffer)?),)*
                }
            }
        }
    };
}

pub mod chat;
pub mod info;
pub mod io;
pub mod packets;
pub mod types;
mod variable;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("failed to write packet id")]
    WritePacketId(#[source] FieldWriteError),

    #[error("failed to write field '{0}'")]
    Field(&'static str, #[source] FieldWriteError),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("unrecognized packet with id {0}")]
    UnrecognizedPacketId(usize),

    #[error("failed to read packet id")]
    ReadPacketId(#[source] FieldReadError),

    #[error("failed to read field '{0}'")]
    Field(&'static str, #[source] FieldReadError),
}

#[derive(Debug, Error)]
pub enum FieldWriteError {
    #[error("json serialization error")]
    Json(#[from] serde_json::Error),

    #[error("write error")]
    WriteError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum FieldReadError {
    #[error("invalid enum id '{0}'")]
    InvalidEnumId(String),

    #[error("variable-sized field too large")]
    VariableTooLarge,

    #[error("json deserialization error")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 conversion error")]
    Utf8(#[from] FromUtf8Error),

    #[error("read error")]
    ReadError(#[from] std::io::Error),
}

pub trait Packet: Sized {
    fn read_from(buffer: &mut dyn Read) -> Result<Self, ReadError>;
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), WriteError>;
}

pub trait PacketField: Sized {
    fn read_from(buffer: &mut dyn Read) -> Result<Self, FieldReadError>;
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError>;

    fn read_from_slice(buffer: &[u8]) -> Result<Self, FieldReadError> {
        Self::read_from(&mut Cursor::new(buffer))
    }

    fn write_to_vec(&self) -> Result<Vec<u8>, FieldWriteError> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf)
    }
}
