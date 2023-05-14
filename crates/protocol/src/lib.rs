use std::{
    io::{Read, Write},
    str::Utf8Error,
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

    #[error("unexpected end-of-input")]
    UnexpectedEoi,

    #[error("failed to convert bytes to string")]
    StrConversion(#[source] Utf8Error),

    #[error("invalid enum variant {key:?}")]
    InvalidEnumVariant { key: String },

    #[error("invalid packet id {0}")]
    InvalidPacketId(i32),
}

pub trait Decodable<'a>: Sized {
    fn decode(r: &mut DecodeBuffer<'a>) -> Result<Self, DecodingError>;
}

pub trait Encoder {
    type Input;

    fn encode(value: Self::Input, w: impl Write) -> Result<(), EncodingError>;
}

pub trait Decoder<'a> {
    type Output;

    fn decode(r: &mut DecodeBuffer<'a>) -> Result<Self::Output, DecodingError>;
}

pub struct DecodeBuffer<'a> {
    position: usize,
    inner: &'a [u8],
}

impl<'a> DecodeBuffer<'a> {
    pub fn new(buffer: &'a [u8]) -> DecodeBuffer {
        DecodeBuffer {
            position: 0,
            inner: buffer,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn advance(&mut self, n: usize) {
        self.position = self.inner.len().min(self.position + n);
    }

    pub fn slice(&self) -> &'a [u8] {
        &self.inner[self.position..]
    }
}

impl<'a> Read for DecodeBuffer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.slice().read(buf)?;
        self.position += len;
        Ok(len)
    }
}
