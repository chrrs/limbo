use protocol_derive::Encodable;

use crate::encodable_packet;

encodable_packet! {
    #[derive(Debug)]
    pub enum ServerStatusPacket {
        0x01 = Pong(StatusPong),
    }
}

#[derive(Debug, Encodable)]
pub struct StatusPong {
    pub payload: u64,
}
