use std::{
    borrow::Cow,
    io::{Cursor, Read, Write},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;
use uuid::Uuid;

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
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        VarInt(self.len() as i32).write_to(buffer)?;
        buffer.write_all(self.as_bytes())?;
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
                    fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), ProtocolError> {
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
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        Ok(buffer.write_u8(*self)?)
    }
}

impl Readable for i8 {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        Ok(buffer.read_i8()?)
    }
}

impl Writable for i8 {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        Ok(buffer.write_i8(*self)?)
    }
}

impl Readable for f32 {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        Ok(buffer.read_f32::<BigEndian>()?)
    }
}

impl Writable for f32 {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        Ok(buffer.write_f32::<BigEndian>(*self)?)
    }
}

impl Readable for f64 {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        Ok(buffer.read_f64::<BigEndian>()?)
    }
}

impl Writable for f64 {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        Ok(buffer.write_f64::<BigEndian>(*self)?)
    }
}

impl Readable for Uuid {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Uuid, ProtocolError> {
        Ok(Uuid::from_u64_pair(
            u64::read_from(buffer)?,
            u64::read_from(buffer)?,
        ))
    }
}

impl Writable for Uuid {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        let (hi, lo) = self.as_u64_pair();
        hi.write_to(buffer)?;
        lo.write_to(buffer)?;
        Ok(())
    }
}

impl Readable for bool {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<bool, ProtocolError> {
        Ok(u8::read_from(buffer)? != 0)
    }
}

impl Writable for bool {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        if *self { 1u8 } else { 0u8 }.write_to(buffer)
    }
}

// TODO: It should be more obvious that these vec's are VarInt-prefixed.
impl<T: Readable> Readable for Vec<T> {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Vec<T>, ProtocolError> {
        let length = VarInt::read_from(buffer)?;
        let mut vec = Vec::with_capacity(length.0 as usize);

        for _ in 0..length.0 {
            vec.push(T::read_from(buffer)?);
        }

        Ok(vec)
    }
}

impl<T: Writable> Writable for Vec<T> {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        VarInt(self.len() as i32).write_to(buffer)?;
        for element in self {
            element.write_to(buffer)?;
        }

        Ok(())
    }
}

pub struct Raw(pub Cow<'static, [u8]>);

impl Raw {
    pub fn new<S: Into<Cow<'static, [u8]>>>(data: S) -> Raw {
        Raw(data.into())
    }
}

impl std::fmt::Debug for Raw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Raw").finish()
    }
}

impl Readable for Raw {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Raw, ProtocolError> {
        let mut vec = Vec::new();
        // TODO: We should use remaining_slice() here, but it's unstable.
        vec.extend_from_slice(&buffer.get_ref()[buffer.position() as usize..]);
        Ok(Raw::new(vec))
    }
}

impl Writable for Raw {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        Ok(buffer.write_all(&self.0)?)
    }
}
