packet! {
    #[derive(Debug)]
    pub enum ClientStatusPacket {
        0x00 = Request { },
        0x01 = Ping {
            payload: i64,
        }
    }
}
