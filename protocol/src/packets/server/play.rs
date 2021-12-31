use crate::chat::Message;

packet! {
    #[derive(Debug)]
    pub enum ServerPlayPacket {
        0x40 = Disconnect {
            reason: Message,
        },
    }
}
