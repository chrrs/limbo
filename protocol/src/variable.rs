use std::{
    fmt::Display,
    io::{Read, Write},
};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{FieldReadError, FieldWriteError, PacketField};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PacketField for VarInt {
    fn read_from(buffer: &mut dyn Read) -> Result<Self, FieldReadError> {
        let mut value = 0;
        let mut length = 0;

        loop {
            let byte = buffer.read_u8()?;
            value |= ((byte & 0x7f) as i32) << (length * 7);
            length += 1;

            if length > 5 {
                break Err(FieldReadError::VariableTooLarge);
            }

            if (byte & 0x80) == 0 {
                break Ok(VarInt(value));
            }
        }
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        let mut value = self.0 as u32;

        loop {
            let part = value as u8;
            value >>= 7;
            if value == 0 {
                buffer.write_u8(part & 0x7f)?;
                break Ok(());
            } else {
                buffer.write_u8(part | 0x80)?;
            }
        }
    }
}
