use std::{
    io::{Read, Write},
    string::FromUtf8Error,
};
use thiserror::Error;

pub mod fields;
pub mod packet;

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("could not write to encoding buffer")]
    Write(#[from] std::io::Error),

    #[error("could not write field '{name}'")]
    Field {
        name: &'static str,
        #[source]
        source: Box<EncodingError>,
    },
}

pub trait Encodable {
    fn encode(&self, w: &mut impl Write) -> Result<(), EncodingError>;
}

#[derive(Debug, Error)]
pub enum DecodingError {
    #[error("could not read from decoding buffer")]
    Read(#[from] std::io::Error),

    #[error("could not read field '{name}'")]
    Field {
        name: &'static str,
        #[source]
        source: Box<DecodingError>,
    },

    #[error("var-int more than 5 bytes in length")]
    VarIntTooLarge,

    #[error("failed to convert bytes to string")]
    StringConversion(#[source] FromUtf8Error),

    #[error("invalid enum variant {key:?}")]
    InvalidEnumVariant { key: String },

    #[error("invalid packet id {0}")]
    InvalidPacketId(i32),
}

pub trait Decodable: Sized {
    fn decode(r: &mut impl Read) -> Result<Self, DecodingError>;
}

pub trait Encoder {
    type Input;

    fn encode(value: Self::Input, w: &mut impl Write) -> Result<(), EncodingError>;
}

pub trait Decoder {
    type Output;

    fn decode(r: &mut impl Read) -> Result<Self::Output, DecodingError>;
}
