use protocol_derive::Decodable;

use crate::{fields::varint::VarIntEncoder, Decodable, Decoder};

#[derive(Debug)]
pub enum ClientStatusPacket {
    Request(StatusRequest),
    Ping(StatusPing),
}

impl Decodable for ClientStatusPacket {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, crate::DecodingError> {
        match VarIntEncoder::decode(r)? {
            0 => Ok(ClientStatusPacket::Request(StatusRequest)),
            1 => StatusPing::decode(r).map(ClientStatusPacket::Ping),
            id => Err(crate::DecodingError::InvalidPacketId(id)),
        }
    }
}

#[derive(Debug)]
pub struct StatusRequest;

#[derive(Debug, Decodable)]
pub struct StatusPing {
    payload: u64,
}
