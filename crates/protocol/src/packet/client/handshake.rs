use crate::{fields::varint::VarIntEncoder, Decodable, DecodeBuffer, Decoder};

#[derive(Debug)]
pub struct Handshake<'a> {
    pub protocol_version: i32,
    pub server_address: &'a str,
    pub server_port: u16,
    pub next_state: i32,
}

impl<'a> Decodable<'a> for Handshake<'a> {
    fn decode(r: &mut DecodeBuffer<'a>) -> Result<Self, crate::DecodingError> {
        Ok(Handshake {
            protocol_version: VarIntEncoder::decode(r).map_err(|e| {
                crate::DecodingError::Field {
                    name: "protocol_version",
                    source: Box::new(e),
                }
            })?,
            server_address: <&str>::decode(r).map_err(|e| crate::DecodingError::Field {
                name: "server_address",
                source: Box::new(e),
            })?,
            server_port: u16::decode(r).map_err(|e| crate::DecodingError::Field {
                name: "server_port",
                source: Box::new(e),
            })?,
            next_state: VarIntEncoder::decode(r).map_err(|e| crate::DecodingError::Field {
                name: "next_state",
                source: Box::new(e),
            })?,
        })
    }
}
