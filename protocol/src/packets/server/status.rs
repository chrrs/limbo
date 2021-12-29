use crate::info::ServerInfo;

packet! {
    #[derive(Debug)]
    pub enum ServerStatusPacket {
        0x00 = Response {
            response: ServerInfo,
        },
        0x01 = Pong {
            payload: i64,
        }
    }
}
