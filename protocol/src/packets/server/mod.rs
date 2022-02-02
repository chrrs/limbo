use std::io::Write;

use crate::{Packet, WriteError};

use self::{login::ServerLoginPacket, play::ServerPlayPacket, status::ServerStatusPacket};

pub mod login;
pub mod play;
pub mod status;

#[derive(Debug)]
pub enum ServerPacket {
    Status(ServerStatusPacket),
    Login(ServerLoginPacket),
    Play(ServerPlayPacket),
}

impl ServerPacket {
    pub fn encode_to(&self, buffer: &mut dyn Write) -> Result<(), WriteError> {
        match self {
            ServerPacket::Status(packet) => packet.write_to(buffer)?,
            ServerPacket::Login(packet) => packet.write_to(buffer)?,
            ServerPacket::Play(packet) => packet.write_to(buffer)?,
        }

        Ok(())
    }
}
