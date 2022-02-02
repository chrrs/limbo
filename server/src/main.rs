use std::{path::Path, sync::Arc};

use anyhow::anyhow;
use client::Client;
use connection::Connection;
use log::{debug, error, info, warn, LevelFilter};
use tokio::{
    net::TcpListener,
    select, signal,
    sync::{broadcast, mpsc::channel, RwLock},
};

use crate::{
    config::{Config, ConfigError},
    shutdown::Shutdown,
};

mod client;
mod config;
mod connection;
mod logging;
mod shutdown;

const CONFIG_PATH: &str = "limbo.toml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "console")]
    console_subscriber::init();

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

    let config = Arc::new(RwLock::new(config));

    let (shutdown, _) = broadcast::channel::<()>(1);
    let (done_send, mut done) = channel::<()>(1);

    loop {
        select! {
            res = listener.accept() => {
                match res {
                    Ok((stream, address)) => {
                        debug!("new connection from {}", address);

                        let config = config.clone();
                        let shutdown = Shutdown::new(shutdown.subscribe());
                        let done = done_send.clone();

                        tokio::spawn(async move {
                            let mut client = Client::new(Connection::new(stream), config, shutdown, done);
                            client.run().await;
                        });
                    },
                    Err(err) => error!("failed to accept connection: {:#}", anyhow!(err)),
                }
            }
            _ = signal::ctrl_c() => break
        }
    }

    info!("shutting down");

    drop(shutdown);
    drop(done_send);
    let _ = done.recv().await;

    Ok(())
}
