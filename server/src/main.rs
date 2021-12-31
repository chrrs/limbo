use std::path::Path;

use anyhow::anyhow;
use client::Client;
use connection::Connection;
use log::{debug, info, warn, LevelFilter};
use protocol::ProtocolError;
use thiserror::Error;
use tokio::net::TcpListener;

use crate::config::{Config, ConfigError};

mod client;
mod config;
mod connection;
mod logging;

const CONFIG_PATH: &str = "limbo.toml";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("connection reset by peer")]
    ConnectionReset,

    #[error("protocol error")]
    Protocol(#[from] ProtocolError),

    #[error("io error")]
    Io(#[from] std::io::Error),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = match config::read(Path::new(CONFIG_PATH)) {
        Ok(config) => {
            logging::init(config.server.log_level)?;
            config
        }
        Err(err) => {
            logging::init(LevelFilter::Info)?;

            let not_found = matches!(err, ConfigError::NotFound);
            warn!("failed to read config file: {:#}", anyhow!(err));

            let config = Config::default();
            if not_found {
                if let Err(err) = config.write(Path::new(CONFIG_PATH)) {
                    warn!("failed to create new config file: {:#}", anyhow!(err));
                } else {
                    info!("intialized default config file ({})", CONFIG_PATH);
                }
            }

            config
        }
    };

    let listener =
        TcpListener::bind(format!("{}:{}", config.server.host, config.server.port)).await?;
    info!(
        "listening on {}:{} for new connections",
        config.server.host, config.server.port
    );

    loop {
        let (stream, address) = listener.accept().await?;
        debug!("new connection from {}", address);

        tokio::spawn(async move {
            let mut client = Client::new(Connection::new(stream));
            client.run().await;
        });
    }
}
