use crate::{chat::Message, io::Raw, types::GameMode, VarInt};

packet! {
    #[derive(Debug)]
    pub enum ServerPlayPacket {
        0x26 = JoinGame {
            entity_id: i32,
            hardcore: bool,
            gamemode: GameMode,
            previous_gamemode: Option<GameMode>,
            world_names: Vec<String>,
            // TODO: Abstract these away better.
            dimension_codec: Raw,
            dimension: Raw,
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
        0x40 = Disconnect {
            reason: Message,
        },
    }
}
