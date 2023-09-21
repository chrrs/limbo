use protocol_derive::Decodable;

use crate::{decodable_packet, fields::varint::VarIntEncoder, Decodable, Decoder};

decodable_packet! {
    #[derive(Debug)]
    pub enum ClientHandshakePacket {
        0x00 = Handshake(Handshake),
    }
}

#[derive(Debug, Decodable)]
pub struct Handshake {
    #[with(VarIntEncoder)]
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: NextState,
}

#[derive(Debug, Clone, Copy)]
pub enum NextState {
    Status,
    Login,
}

impl Decodable for NextState {
    fn decode(r: &mut impl std::io::Read) -> Result<Self, crate::DecodingError> {
        match VarIntEncoder::decode(r)? {
            1 => Ok(NextState::Status),
            2 => Ok(NextState::Login),
            key => Err(crate::DecodingError::InvalidEnumVariant {
                key: format!("{key}"),
            }),
        }
    }
}
