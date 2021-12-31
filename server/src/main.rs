use client::Client;
use connection::Connection;
use log::{debug, info, LevelFilter};
use protocol::ProtocolError;
use thiserror::Error;
use tokio::net::TcpListener;

mod client;
mod connection;
mod logging;

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
    logging::init(LevelFilter::Debug)?;

    let listener = TcpListener::bind("0.0.0.0:25565").await?;
    info!("listening on 0.0.0.0:25565 for new connections");

    loop {
        let (stream, address) = listener.accept().await?;
        debug!("new connection from {}", address);

        tokio::spawn(async move {
            let mut client = Client::new(Connection::new(stream));
            client.run().await;
        });
    }
}
