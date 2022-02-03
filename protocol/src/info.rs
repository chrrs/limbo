use std::{
    borrow::Cow,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::{chat::Message, FieldReadError, FieldWriteError, PacketField};

pub const VERSION: VersionInfo = VersionInfo {
    name: Cow::Borrowed("1.18.1"),
    protocol: 757,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    version: VersionInfo,
    players: Option<ServerPlayerInfo>,
    description: Message,

    #[serde(skip_serializing_if = "Option::is_none")]
    favicon: Option<String>,
}

impl ServerInfo {
    pub fn new(
        version: VersionInfo,
        players: Option<ServerPlayerInfo>,
        motd: Message,
        favicon: Option<&[u8]>,
    ) -> ServerInfo {
        ServerInfo {
            version,
            players,
            description: motd,
            favicon: favicon.map(|data| format!("data:image/png;base64,{}", base64::encode(data))),
        }
    }
}

impl PacketField for ServerInfo {
    fn read_from(buffer: &mut dyn Read) -> Result<ServerInfo, FieldReadError> {
        Ok(serde_json::from_str(&String::read_from(buffer)?)?)
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        serde_json::to_string(self)?.write_to(buffer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerPlayerInfo {
    max: isize,
    online: isize,
}

impl ServerPlayerInfo {
    pub fn simple(online: isize, max: isize) -> ServerPlayerInfo {
        ServerPlayerInfo { online, max }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: Cow<'static, str>,
    pub protocol: usize,
}
