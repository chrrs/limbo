use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{Decoder, DecodingError, Encoder, EncodingError};

pub struct VarIntEncoder;

impl Encoder for VarIntEncoder {
    type Input = i32;

    fn encode(value: Self::Input, mut w: impl std::io::Write) -> Result<(), EncodingError> {
        let mut value = value as u32;

        loop {
            let part = value as u8;
            value >>= 7;
            if value == 0 {
                w.write_u8(part & 0x7f)?;
                break Ok(());
            } else {
                w.write_u8(part | 0x80)?;
            }
        }
    }
}

impl Decoder for VarIntEncoder {
    type Output = i32;

    fn decode(mut r: impl std::io::Read) -> Result<Self::Output, DecodingError> {
        let mut value = 0;
        let mut length = 0;

        loop {
            let byte = r.read_u8()?;
            value |= ((byte & 0x7f) as u32) << (length * 7);
            length += 1;

            if (byte & 0x80) == 0 {
                break Ok(value as i32);
            }

            if length > 5 {
                break Err(DecodingError::VarIntTooLarge);
            }
        }
    }
}
