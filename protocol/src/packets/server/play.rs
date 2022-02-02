use crate::{
    chat::Message,
    io::{RawBytes, VarIntPrefixedVec},
    types::{GameMode, Position},
    VarInt,
};

packet! {
    #[derive(Debug)]
    pub enum ServerPlayPacket {
        0x18 = PluginMessage {
            channel: String,
            data: RawBytes,
        },
        0x21 = KeepAlive {
            id: u64,
        },
        0x26 = JoinGame {
            entity_id: i32,
            hardcore: bool,
            gamemode: GameMode,
            previous_gamemode: Option<GameMode>,
            world_names: VarIntPrefixedVec<String>,
            // TODO: Abstract these away better.
            dimension_codec: RawBytes,
            dimension: RawBytes,
            world_name: String,
            hashed_seed: i64,
            max_players: VarInt,
            view_distance: VarInt,
            simulation_distance: VarInt,
            reduced_debug_info: bool,
            enable_respawn_screen: bool,
            debug: bool,
            flat: bool,
        },
        0x1a = Disconnect {
            reason: Message,
        },
        0x38 = PlayerPositionAndLook {
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            pitch: f32,
            flags: u8,
            teleport_id: VarInt,
            dismount_vehicle: bool,
        },
        0x4b = SpawnPosition {
            location: Position,
            angle: f32,
        }
    }
}
