use std::io::{Read, Write};
use thiserror::Error;

pub mod fields;

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
    fn encode(&self, w: impl Write) -> Result<(), EncodingError>;
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
}

pub trait Decodable: Sized {
    fn decode(r: impl Read) -> Result<Self, DecodingError>;
}

pub trait Encoder {
    type Input;

    fn encode(value: Self::Input, w: impl Write) -> Result<(), EncodingError>;
}

pub trait Decoder {
    type Output;

    fn decode(r: impl Read) -> Result<Self::Output, DecodingError>;
}
