use serde_derive::Serialize;

pub const VERSION: VersionInfo = VersionInfo {
    name: "1.18.1",
    protocol: 757,
};

#[derive(Serialize)]
pub struct ServerInfo {
    version: VersionInfo,
    players: PlayerInfo,
    description: Motd,
}

impl ServerInfo {
    pub fn new(version: VersionInfo, players: PlayerInfo, motd: Motd) -> ServerInfo {
        ServerInfo {
            version,
            players,
            description: motd,
        }
    }
}

#[derive(Serialize)]
pub struct PlayerInfo {
    max: isize,
    online: isize,
}

impl PlayerInfo {
    pub fn simple(online: isize, max: isize) -> PlayerInfo {
        PlayerInfo { online, max }
    }
}

#[derive(Serialize)]
pub struct Motd {
    text: String,
}

impl Motd {
    pub fn new(motd: String) -> Motd {
        Motd { text: motd }
    }
}

#[derive(Serialize)]
pub struct VersionInfo {
    name: &'static str,
    protocol: usize,
}
