use std::{
    borrow::Cow,
    io::{Read, Write},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;
use uuid::Uuid;

use crate::{FieldReadError, FieldWriteError, PacketField, VarInt};

impl PacketField for String {
    fn read_from(buffer: &mut dyn Read) -> Result<Self, FieldReadError> {
        let length = VarInt::read_from(buffer)?;
        let mut string_buffer = vec![0; length.0 as usize];
        buffer.read_exact(&mut string_buffer)?;
        Ok(String::from_utf8(string_buffer)?)
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        VarInt(self.len() as i32).write_to(buffer)?;
        buffer.write_all(self.as_bytes())?;
        Ok(())
    }
}

macro_rules! impl_int {
    ($($typ:ident),+) => {
        $(
            paste! {
                impl PacketField for $typ {
                    fn read_from(buffer: &mut dyn Read) -> Result<Self, FieldReadError> {
                        Ok(buffer.[<read_ $typ>]::<BigEndian>()?)
                    }

                    fn write_to(&self, buffer: &mut dyn std::io::Write) -> Result<(), FieldWriteError> {
                        Ok(buffer.[<write_ $typ>]::<BigEndian>(*self)?)
                    }
                }
            }
        )+
    };
}

impl_int!(u16, u32, u64, i16, i32, i64, f32, f64);

impl PacketField for u8 {
    fn read_from(buffer: &mut dyn Read) -> Result<Self, FieldReadError> {
        Ok(buffer.read_u8()?)
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        Ok(buffer.write_u8(*self)?)
    }
}

impl PacketField for i8 {
    fn read_from(buffer: &mut dyn Read) -> Result<Self, FieldReadError> {
        Ok(buffer.read_i8()?)
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        Ok(buffer.write_i8(*self)?)
    }
}

impl PacketField for Uuid {
    fn read_from(buffer: &mut dyn Read) -> Result<Uuid, FieldReadError> {
        Ok(Uuid::from_u64_pair(
            u64::read_from(buffer)?,
            u64::read_from(buffer)?,
        ))
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        let (hi, lo) = self.as_u64_pair();
        hi.write_to(buffer)?;
        lo.write_to(buffer)?;
        Ok(())
    }
}

impl PacketField for bool {
    fn read_from(buffer: &mut dyn Read) -> Result<bool, FieldReadError> {
        Ok(u8::read_from(buffer)? != 0)
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        if *self { 1u8 } else { 0u8 }.write_to(buffer)
    }
}

#[derive(Debug)]
pub struct VarIntPrefixedVec<T>(pub Vec<T>);

impl<T: PacketField> PacketField for VarIntPrefixedVec<T> {
    fn read_from(buffer: &mut dyn Read) -> Result<VarIntPrefixedVec<T>, FieldReadError> {
        let length = VarInt::read_from(buffer)?;
        let mut vec = Vec::with_capacity(length.0 as usize);

        for _ in 0..length.0 {
            vec.push(T::read_from(buffer)?);
        }

        Ok(VarIntPrefixedVec(vec))
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        VarInt(self.0.len() as i32).write_to(buffer)?;
        for element in &self.0 {
            element.write_to(buffer)?;
        }

        Ok(())
    }
}

pub struct RawBytes(pub Cow<'static, [u8]>);

impl RawBytes {
    pub fn new<S: Into<Cow<'static, [u8]>>>(data: S) -> RawBytes {
        RawBytes(data.into())
    }
}

impl std::fmt::Debug for RawBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RawBytes").finish()
    }
}

impl PacketField for RawBytes {
    fn read_from(buffer: &mut dyn Read) -> Result<RawBytes, FieldReadError> {
        let mut vec = Vec::new();
        buffer.read_to_end(&mut vec)?;
        Ok(RawBytes::new(vec))
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        Ok(buffer.write_all(&self.0)?)
    }
}

#[derive(Debug)]
pub struct BooleanPrefixedOption<T>(pub Option<T>);

impl<T: PacketField> PacketField for BooleanPrefixedOption<T> {
    fn read_from(buffer: &mut dyn Read) -> Result<BooleanPrefixedOption<T>, FieldReadError> {
        if bool::read_from(buffer)? {
            Ok(BooleanPrefixedOption(Some(T::read_from(buffer)?)))
        } else {
            Ok(BooleanPrefixedOption(None))
        }
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        self.0.is_some().write_to(buffer)?;
        if let Some(value) = &self.0 {
            value.write_to(buffer)?;
        }

        Ok(())
    }
}
