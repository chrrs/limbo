use connection::Connection;
use tokio::net::TcpListener;

use crate::protocol::packets::State;

mod connection;
mod protocol;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut connection = Connection::new(stream);
            println!("- new connection made");
            println!("{:?}", connection.read_packet().await);
            connection.state = State::Status;
            println!("{:?}", connection.read_packet().await);
        });
    }
}
