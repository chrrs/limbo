use uuid::Uuid;

use crate::{chat::Message, VarInt};

packet! {
    #[derive(Debug)]
    pub enum ServerLoginPacket {
        0x00 = Disconnect {
            reason: Message,
        },
        0x02 = Success {
            uuid: Uuid,
            name: String,
        },
        0x03 = SetCompression {
            threshold: VarInt,
        },
    }
}
