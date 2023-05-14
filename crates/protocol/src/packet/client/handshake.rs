use protocol_derive::Decodable;

use crate::{fields::varint::VarIntEncoder, Decodable, Decoder};

#[derive(Debug)]
pub enum HandshakePacket<'a> {
    Handshake(Handshake<'a>),
}

impl<'a> Decodable<'a> for HandshakePacket<'a> {
    fn decode(r: &mut crate::DecodeBuffer<'a>) -> Result<Self, crate::DecodingError> {
        match VarIntEncoder::decode(r)? {
            0 => Handshake::decode(r).map(HandshakePacket::Handshake),
            id => Err(crate::DecodingError::InvalidPacketId(id)),
        }
    }
}

#[derive(Debug, Decodable)]
pub struct Handshake<'a> {
    #[with(VarIntEncoder)]
    pub protocol_version: i32,
    pub server_address: &'a str,
    pub server_port: u16,
    pub next_state: NextState,
}

#[derive(Debug, Clone, Copy)]
pub enum NextState {
    Status,
    Login,
}

impl Decodable<'_> for NextState {
    fn decode(r: &mut crate::DecodeBuffer) -> Result<Self, crate::DecodingError> {
        match VarIntEncoder::decode(r)? {
            1 => Ok(NextState::Status),
            2 => Ok(NextState::Login),
            key => Err(crate::DecodingError::InvalidEnumVariant {
                key: format!("{key}"),
            }),
        }
    }
}
