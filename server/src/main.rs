use client::Client;
use connection::Connection;
use protocol::ProtocolError;
use thiserror::Error;
use tokio::net::TcpListener;

mod client;
mod connection;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("connection reset by peer")]
    ConnectionReset,

    #[error("failed to process packet")]
    Protocol(#[from] ProtocolError),

    #[error("io error")]
    Io(#[from] std::io::Error),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut client = Client::new(Connection::new(stream));
            client.run().await;
        });
    }
}
