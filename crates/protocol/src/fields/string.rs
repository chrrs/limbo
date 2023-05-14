use crate::{Decodable, Decoder, Encodable, Encoder};

use super::varint::VarIntEncoder;

impl Encodable for &'_ str {
    fn encode(&self, w: &mut impl std::io::Write) -> Result<(), crate::EncodingError> {
        VarIntEncoder::encode(self.len() as i32, w).map_err(|e| crate::EncodingError::Field {
            name: "length",
            source: Box::new(e),
        })?;

        w.write_all(self.as_bytes())
            .map_err(crate::EncodingError::Write)?;

        Ok(())
    }
}

impl Decodable for String {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, crate::DecodingError> {
        let length = VarIntEncoder::decode(r).map_err(|e| crate::DecodingError::Field {
            name: "length",
            source: Box::new(e),
        })? as usize;

        let mut buf = vec![0; length];
        r.read_exact(&mut buf)?;

        String::from_utf8(buf).map_err(crate::DecodingError::StringConversion)
    }
}
