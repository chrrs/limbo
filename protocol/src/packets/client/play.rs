use crate::io::Raw;

packet! {
    #[derive(Debug)]
    pub enum ClientPlayPacket {
        0x0a = PluginMessage {
            channel: String,
            data: Raw,
        },
    }
}
