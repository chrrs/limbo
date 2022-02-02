use std::io::Read;

use crate::{Packet, ReadError};

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
    pub fn decode(state: State, buffer: &mut dyn Read) -> Result<ClientPacket, ReadError> {
        match state {
            State::Handshake => Ok(ClientPacket::Handshake(ClientHandshakePacket::read_from(
                buffer,
            )?)),
            State::Status => Ok(ClientPacket::Status(ClientStatusPacket::read_from(buffer)?)),
            State::Login => Ok(ClientPacket::Login(ClientLoginPacket::read_from(buffer)?)),
            State::Play => Ok(ClientPacket::Play(ClientPlayPacket::read_from(buffer)?)),
        }
    }
}
