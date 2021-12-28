use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;

use super::{ProtocolError, Readable, VarInt, Writable};

impl Readable for String {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        let length = VarInt::read_from(buffer)?;
        let mut string_buffer = vec![0; length.0 as usize];
        buffer.read_exact(&mut string_buffer)?;
        Ok(String::from_utf8(string_buffer)?)
    }
}

impl Writable for String {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError> {
        VarInt(self.len() as i32).write_to(buffer)?;
        buffer.extend(self.bytes());
        Ok(())
    }
}

macro_rules! impl_int {
    ($($typ:ident),+) => {
        $(
            paste! {
                impl Readable for $typ {
                    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
                        Ok(buffer.[<read_ $typ>]::<BigEndian>()?)
                    }
                }

                impl Writable for $typ {
                    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError> {
                        Ok(buffer.[<write_ $typ>]::<BigEndian>(*self)?)
                    }
                }
            }
        )+
    };
}

impl_int!(u16, u32, u64, i16, i32, i64);

impl Readable for u8 {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        Ok(buffer.read_u8()?)
    }
}

impl Writable for u8 {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError> {
        Ok(buffer.write_u8(*self)?)
    }
}

impl Readable for i8 {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        Ok(buffer.read_i8()?)
    }
}

impl Writable for i8 {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError> {
        Ok(buffer.write_i8(*self)?)
    }
}
