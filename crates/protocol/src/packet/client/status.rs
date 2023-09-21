use protocol_derive::Decodable;

use crate::decodable_packet;

decodable_packet! {
    #[derive(Debug)]
    pub enum ClientStatusPacket {
        0x00 = Request(StatusRequest),
        0x01 = Ping(StatusPing),
    }
}

#[derive(Debug, Decodable)]
pub struct StatusRequest;

#[derive(Debug, Decodable)]
pub struct StatusPing {
    payload: u64,
}
