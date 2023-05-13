use crate::{Decodable, DecodeBuffer, Decoder, Encodable, Encoder};

use super::varint::VarIntEncoder;

impl Encodable for &'_ str {
    fn encode(&self, mut w: impl std::io::Write) -> Result<(), crate::EncodingError> {
        VarIntEncoder::encode(self.len() as i32, &mut w).map_err(|e| {
            crate::EncodingError::Field {
                name: "length",
                source: Box::new(e),
            }
        })?;

        w.write_all(self.as_bytes())
            .map_err(crate::EncodingError::Write)?;

        Ok(())
    }
}

impl<'a> Decodable<'a> for &'a str {
    fn decode(r: &mut DecodeBuffer<'a>) -> Result<Self, crate::DecodingError> {
        let length = VarIntEncoder::decode(r).map_err(|e| crate::DecodingError::Field {
            name: "length",
            source: Box::new(e),
        })? as usize;

        let slice = r.slice();
        if slice.len() < length {
            return Err(crate::DecodingError::UnexpectedEoi);
        }

        r.advance(length);
        core::str::from_utf8(&slice[..length]).map_err(crate::DecodingError::StrConversion)
    }
}
