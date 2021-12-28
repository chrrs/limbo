use std::io::Cursor;

use byteorder::ReadBytesExt;

use super::{Error, Readable, Writable};

pub struct VarInt(pub i32);

impl Readable for VarInt {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        let mut value: i32 = 0;
        let mut length = 0;

        loop {
            value |= ((buffer.read_u8()? & 0x7f) as i32) << (length * 7);
            length += 1;

            if length > 5 {
                break Err(Error::VariableTooLarge);
            }

            if (value & 0x80) == 0 {
                break Ok(VarInt(value));
            }
        }
    }
}

impl Writable for VarInt {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
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
