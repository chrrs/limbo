packet! {
    #[derive(Debug)]
    pub enum ClientLoginPacket {
        0x00 = Start {
            name: String,
        },
    }
}
