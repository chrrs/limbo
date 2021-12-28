use std::io::{Cursor, Read};

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
