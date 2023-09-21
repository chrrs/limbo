use network::Connection;
use protocol::packet::client::handshake::ClientHandshakePacket;
use tokio::{net::TcpListener, select, signal};
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");

    let addr = "0.0.0.0:25565";
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to open TCP listener");

    info!("listening on {addr} for new connections");

    loop {
        select! {
            res = listener.accept() => {
                match res {
                    Ok((stream, address)) => {
                        debug!("new connection from {address}");
                        let mut connection = Connection::new(stream);

                        match connection.receive_packet::<ClientHandshakePacket>().await {
                            Ok(_packet) => {},
                            Err(err) => {
                                error!("error while receiving packet: {err}");
                                break;
                            }
                        }
                    },
                    Err(err) => error!("failed to accept connection: {err}"),
                }
            }
            _ = signal::ctrl_c() => break
        }
    }

    info!("shutting down");
}
