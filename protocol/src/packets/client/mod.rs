use std::io::Cursor;

use crate::{ProtocolError, Readable};

use self::handshake::ClientHandshakePacket;
use self::login::ClientLoginPacket;
use self::play::ClientPlayPacket;
use self::status::ClientStatusPacket;

use super::State;

pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

#[derive(Debug)]
pub enum ClientPacket {
    Handshake(ClientHandshakePacket),
    Status(ClientStatusPacket),
    Login(ClientLoginPacket),
    Play(ClientPlayPacket),
}

impl ClientPacket {
    pub fn decode(state: State, cursor: &mut Cursor<&[u8]>) -> Result<ClientPacket, ProtocolError> {
        match state {
            State::Handshake => Ok(ClientPacket::Handshake(ClientHandshakePacket::read_from(
                cursor,
            )?)),
            State::Status => Ok(ClientPacket::Status(ClientStatusPacket::read_from(cursor)?)),
            State::Login => Ok(ClientPacket::Login(ClientLoginPacket::read_from(cursor)?)),
            State::Play => Ok(ClientPacket::Play(ClientPlayPacket::read_from(cursor)?)),
        }
    }
}
