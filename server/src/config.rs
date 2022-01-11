use std::{fs::OpenOptions, io::Write, path::Path};

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("file not found")]
    NotFound,

    #[error("io error")]
    Io(#[from] std::io::Error),

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
    pub max_players: isize,
    pub hide_player_count: bool,
    pub motd: String,
}

impl Default for Info {
    fn default() -> Info {
        Info {
            max_players: -1,
            hide_player_count: false,
            motd: "A Limbo Server".to_string(),
        }
    }
}

pub fn read(path: &Path) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound);
    }

    let bytes = std::fs::read(path)?;
    let config = toml::from_slice(&bytes)?;
    Ok(config)
}
