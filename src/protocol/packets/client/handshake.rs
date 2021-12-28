use crate::protocol::{Readable, VarInt, Writable};

#[derive(Debug)]
pub struct ClientHandshakePacket {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarInt,
}

impl Readable for ClientHandshakePacket {
    fn read_from(
        buffer: &mut std::io::Cursor<&[u8]>,
    ) -> Result<Self, crate::protocol::ProtocolError> {
        Ok(Self {
            protocol_version: VarInt::read_from(buffer)?,
            server_address: String::read_from(buffer)?,
            server_port: u16::read_from(buffer)?,
            next_state: VarInt::read_from(buffer)?,
        })
    }
}

impl Writable for ClientHandshakePacket {
    fn write_to(&self, buffer: &mut Vec<u8>) -> Result<(), crate::protocol::ProtocolError> {
        self.protocol_version.write_to(buffer)?;
        self.server_address.write_to(buffer)?;
        self.server_port.write_to(buffer)?;
        self.next_state.write_to(buffer)?;
        Ok(())
    }
}
