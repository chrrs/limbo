use crate::{io::RawBytes, VarInt};

packet! {
    #[derive(Debug)]
    pub enum ClientPlayPacket {
        0x00 = TeleportConfirm {
            id: VarInt,
        },
        0x05 = ClientSettings {
            locale: String,
            view_distance: i8,
            chat_mode: VarInt,
            chat_colors: bool,
            displayed_skin_parts: u8,
            main_hand: VarInt,
            text_filtering: bool,
            allow_server_listings: bool,
        },
        0x0a = PluginMessage {
            channel: String,
            data: RawBytes,
        },
        0x0f = KeepAlive {
            id: u64,
        },
        0x11 = PlayerPosition {
            x: f64,
            y: f64,
            z: f64,
            on_ground: bool,
        },
        0x12 = PlayerPositionAndRotation {
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            pitch: f32,
            on_ground: bool,
        },
    }
}
