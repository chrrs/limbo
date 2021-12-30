use client::Client;
use connection::Connection;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::{debug, info};
use protocol::ProtocolError;
use thiserror::Error;
use tokio::net::TcpListener;

mod client;
mod connection;

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
    let colors = ColoredLevelConfig::new()
        .trace(Color::Cyan)
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] \x1b[0m{}",
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()?;

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
