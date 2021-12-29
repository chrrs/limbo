use std::io::Write;

use crate::protocol::{ProtocolError, Writable};

use self::status::ServerStatusPacket;

pub mod status;

#[derive(Debug)]
pub enum ServerPacket {
    Status(ServerStatusPacket),
}

impl ServerPacket {
    pub fn encode_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        match self {
            ServerPacket::Status(packet) => packet.write_to(buffer)?,
        }

        Ok(())
    }
}
