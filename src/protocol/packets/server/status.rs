use limbo_derive::Packet;

#[derive(Packet)]
pub struct ServerResponsePacket {
    pub response: String,
}
