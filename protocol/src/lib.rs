use std::{
    borrow::Cow,
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

        impl crate::Readable for $name {
            fn read_from(buffer: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::ProtocolError> {
                match crate::VarInt::read_from(buffer)?.0 {
                    $(
                        $id => Ok(Self::$packet {
                            $(
                                $field: $typ::read_from(buffer)?,
                            )*
                        }),
                    )*
                    id => Err(crate::ProtocolError::InvalidPacketId(id)),
                }
            }
        }

        impl crate::Writable for $name {
            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::ProtocolError> {
                match self {
                    $(
                        Self::$packet { $($field),* } => {
                            crate::VarInt($id).write_to(buffer)?;
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
            $($variant:ident = $class:ident ( $arg:expr )),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }

        impl crate::Readable for $name {
            fn read_from(buffer: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::ProtocolError> {
                let value = $super::read_from(buffer)?;
                match value {
                    $($class($arg) => Ok(Self::$variant),)*
                    id => Err(crate::ProtocolError::InvalidEnumVariant { id: std::borrow::Cow::Owned(format!("{:?}", id)), name: std::borrow::Cow::Borrowed(stringify!($name)) })
                }
            }
        }

        impl crate::Writable for $name {
            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::ProtocolError> {
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

        impl crate::Readable for $name {
            fn read_from(buffer: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::ProtocolError> {
                let value = $super::read_from(buffer)?;
                match value {
                    $($id => Ok(Self::$variant),)*
                    id => Err(crate::ProtocolError::InvalidEnumVariant { id: std::borrow::Cow::Owned(format!("{:?}", id)), name: std::borrow::Cow::Borrowed(stringify!($name)) })
                }
            }
        }

        impl crate::Writable for $name {
            fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), crate::ProtocolError> {
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

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("VarInt or VarLong spanning too many bytes")]
    VariableTooLarge,

    #[error("string contains non-UTF8 characters")]
    InvalidString(#[from] FromUtf8Error),

    #[error("packet with id {0} not recognized")]
    InvalidPacketId(i32),

    #[error("enum variant with id {id} does not exist for {name}")]
    InvalidEnumVariant {
        // TODO: Maybe this should be something else than a string?
        id: Cow<'static, str>,
        name: Cow<'static, str>,
    },

    #[error("json error")]
    Json(#[from] serde_json::Error),

    #[error("io error")]
    Io(#[from] std::io::Error),
}

pub trait Readable: Sized {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError>;

    fn decode(buffer: &[u8]) -> Result<Self, ProtocolError> {
        Self::read_from(&mut Cursor::new(buffer))
    }
}

pub trait Writable {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError>;

    fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf)
    }
}
