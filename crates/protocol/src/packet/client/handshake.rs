use protocol_derive::Decodable;

use crate::fields::varint::VarIntEncoder;

#[derive(Debug, Decodable)]
pub struct Handshake<'a> {
    #[with(VarIntEncoder)]
    pub protocol_version: i32,
    pub server_address: &'a str,
    pub server_port: u16,
    #[with(VarIntEncoder)]
    pub next_state: i32,
}
