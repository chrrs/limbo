use uuid::Uuid;

use crate::chat::Message;

packet! {
    #[derive(Debug)]
    pub enum ServerLoginPacket {
        0x00 = Disconnect {
            reason: Message,
        },
        0x02 = Success {
            uuid: Uuid,
            name: String,
        }
    }
}
