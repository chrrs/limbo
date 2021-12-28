use crate::protocol::{ProtocolError, Writable};

use self::status::ServerStatusPacket;

pub mod status;

#[derive(Debug)]
pub enum ServerPacket {
    Status(ServerStatusPacket),
}

impl ServerPacket {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Vec::new();

        match self {
            ServerPacket::Status(packet) => packet.write_to(&mut buffer)?,
        }

        Ok(buffer)
    }
}
