use crate::chat::Message;

packet! {
    #[derive(Debug)]
    pub enum ServerLoginPacket {
        0x00 = Disconnect {
            reason: Message,
        },
    }
}
