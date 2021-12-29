use crate::{packets::State, VarInt};

packet! {
    #[derive(Debug)]
    pub enum ClientHandshakePacket {
        0x00 = Handshake {
            protocol_version: VarInt,
            server_address: String,
            server_port: u16,
            next_state: State,
        },
    }
}
