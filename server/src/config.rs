use std::{fs::OpenOptions, io::Write, path::Path, str::Utf8Error};

use anyhow::anyhow;
use log::{warn, LevelFilter};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("file not found")]
    NotFound,

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("file not correctly utf-8 encoded")]
    InvalidUtf8(#[from] Utf8Error),

    #[error("deserialization error")]
    DeserializationError(#[from] toml::de::Error),

    #[error("serialization error")]
    SerializationError(#[from] toml::ser::Error),
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub server: Server,
    pub info: Info,
}

impl Config {
    pub fn write(&self, path: &Path) -> Result<(), ConfigError> {
        let out = toml::to_string_pretty(self)?;
        let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
        file.write_all(out.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub log_level: LevelFilter,
}

impl Default for Server {
    fn default() -> Server {
        Server {
            host: "0.0.0.0".to_string(),
            port: 25565,
            log_level: LevelFilter::Info,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub hidden: bool,
    pub max_players: isize,
    pub hide_player_count: bool,
    pub motd: String,
    pub name: String,
    pub icon: String,

    #[serde(skip_serializing, skip_deserializing)]
    icon_cache: Option<Vec<u8>>,
}

impl Info {
    pub fn icon(&mut self) -> Option<&[u8]> {
        if let Some(ref icon) = self.icon_cache {
            return if icon.is_empty() { None } else { Some(icon) };
        }

        let path = Path::new(&self.icon);

        if !path.exists() {
            warn!("server favicon file does not exist");
            self.icon_cache = Some(Vec::new());
            return None;
        }

        match std::fs::read(path) {
            Ok(bytes) => {
                self.icon_cache = Some(bytes);
                self.icon_cache.as_ref().map(|v| &v[..])
            }
            Err(err) => {
                warn!("failed to read server favicon file: {:#}", anyhow!(err));
                self.icon_cache = Some(Vec::new());
                None
            }
        }
    }
}

impl Default for Info {
    fn default() -> Info {
        Info {
            hidden: false,
            max_players: -1,
            hide_player_count: false,
            motd: "A Limbo Server".to_string(),
            name: "Limbo".to_string(),
            icon: "icon.png".to_string(),
            icon_cache: None,
        }
    }
}

pub fn read(path: &Path) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound);
    }

    let bytes = std::fs::read(path)?;
    let string = core::str::from_utf8(&bytes)?;
    let config = toml::from_str(string)?;
    Ok(config)
}
