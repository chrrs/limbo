use std::io::Cursor;

use crate::protocol::{ProtocolError, Readable};

use self::handshake::ClientHandshakePacket;

use super::State;

pub mod handshake;

#[derive(Debug)]
pub enum ClientPacket {
    Handshake(ClientHandshakePacket),
    // Status(ClientStatusPacket),
}

impl ClientPacket {
    pub fn decode(state: State, cursor: &mut Cursor<&[u8]>) -> Result<ClientPacket, ProtocolError> {
        match state {
            State::Handshake => Ok(ClientPacket::Handshake(ClientHandshakePacket::read_from(
                cursor,
            )?)),
            State::Status => Err(ProtocolError::InvalidPacketId(-1)),
            State::Login => Err(ProtocolError::InvalidPacketId(-1)),
            State::Play => Err(ProtocolError::InvalidPacketId(-1)),
        }
    }
}
