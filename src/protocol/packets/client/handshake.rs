use limbo_derive::Packet;

use crate::protocol::VarInt;

#[derive(Packet, Debug)]
pub struct ClientHandshakePacket {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarInt,
}
