use std::{fmt::Display, io::Cursor};

use byteorder::ReadBytesExt;

use super::{ProtocolError, Readable, Writable};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Readable for VarInt {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, ProtocolError> {
        let mut value = 0;
        let mut length = 0;

        loop {
            let byte = buffer.read_u8()?;
            value |= ((byte & 0x7f) as i32) << (length * 7);
            length += 1;

            if length > 5 {
                break Err(ProtocolError::VariableTooLarge);
            }

            if (byte & 0x80) == 0 {
                break Ok(VarInt(value));
            }
        }
    }
}

impl Writable for VarInt {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), ProtocolError> {
        let mut value = self.0;

        loop {
            if (value & 0x80) == 0 {
                buffer.push(value as u8);
                break Ok(());
            }

            buffer.push(((value & 0x7f) | 0x80) as u8);
            value >>= 7;
        }
    }
}
