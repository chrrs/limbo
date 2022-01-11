use crate::io::Raw;

packet! {
    #[derive(Debug)]
    pub enum ClientPlayPacket {
        0x0a = PluginMessage {
            channel: String,
            data: Raw,
        },
        0x11 = PlayerPosition {
            x: f64,
            y: f64,
            z: f64,
            on_ground: bool,
        }
    }
}
