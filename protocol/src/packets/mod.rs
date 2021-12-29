use super::VarInt;

packet_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum State: VarInt {
        Handshake = VarInt(0),
        Status = VarInt(1),
        Login = VarInt(2),
        Play = VarInt(3),
    }
}

pub mod client;
pub mod server;
