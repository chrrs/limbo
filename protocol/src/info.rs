use std::{
    borrow::Cow,
    io::{Cursor, Write},
};

use serde::{Deserialize, Serialize};

use crate::{chat::Message, ProtocolError, Readable, Writable};

pub const VERSION: VersionInfo = VersionInfo {
    name: Cow::Borrowed("1.18.1"),
    protocol: 757,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    version: VersionInfo,
    players: Option<PlayerInfo>,
    description: Message,
}

impl ServerInfo {
    pub fn new(version: VersionInfo, players: Option<PlayerInfo>, motd: Message) -> ServerInfo {
        ServerInfo {
            version,
            players,
            description: motd,
        }
    }
}

impl Readable for ServerInfo {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<ServerInfo, ProtocolError> {
        Ok(serde_json::from_str(&String::read_from(buffer)?)?)
    }
}

impl Writable for ServerInfo {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        serde_json::to_string(self)?.write_to(buffer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerInfo {
    max: isize,
    online: isize,
}

impl PlayerInfo {
    pub fn simple(online: isize, max: isize) -> PlayerInfo {
        PlayerInfo { online, max }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: Cow<'static, str>,
    pub protocol: usize,
}
