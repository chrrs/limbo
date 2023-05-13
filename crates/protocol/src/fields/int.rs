use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use paste::paste;

use crate::{Decodable, DecodeBuffer, Encodable};

macro_rules! impl_int {
    ($($typ:ident)+) => {
        $(paste! {
            impl Encodable for $typ {
                fn encode(&self, mut w: impl std::io::Write) -> Result<(), crate::EncodingError> {
                    w.[<write_ $typ>]::<BigEndian>(*self).map_err(crate::EncodingError::Write)
                }
            }

            impl Decodable<'_> for $typ {
                fn decode(r: &mut DecodeBuffer) -> Result<Self, crate::DecodingError> {
                    r.[<read_ $typ>]::<BigEndian>().map_err(crate::DecodingError::Read)
                }
            }
       })+
    };
}

impl_int!(u16 i16 u32 i32 u64 i64 f32 f64);

impl Encodable for u8 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<(), crate::EncodingError> {
        w.write_u8(*self).map_err(crate::EncodingError::Write)
    }
}

impl Decodable<'_> for u8 {
    fn decode(r: &mut DecodeBuffer) -> Result<Self, crate::DecodingError> {
        r.read_u8().map_err(crate::DecodingError::Read)
    }
}

impl Encodable for i8 {
    fn encode(&self, mut w: impl std::io::Write) -> Result<(), crate::EncodingError> {
        w.write_i8(*self).map_err(crate::EncodingError::Write)
    }
}

impl Decodable<'_> for i8 {
    fn decode(r: &mut DecodeBuffer) -> Result<Self, crate::DecodingError> {
        r.read_i8().map_err(crate::DecodingError::Read)
    }
}
