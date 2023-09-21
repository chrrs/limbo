use network::Connection;
use protocol::{
    fields::varint::VarIntEncoder,
    packet::client::{handshake::ClientHandshakePacket, status::ClientStatusPacket},
    Encodable, Encoder,
};
use tokio::{net::TcpListener, select, signal};
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug)]
struct TestResponse;

impl Encodable for TestResponse {
    fn encode(&self, w: &mut impl std::io::Write) -> Result<(), protocol::EncodingError> {
        VarIntEncoder::encode(0, w)?;
        r#"{"version":{"name":"Limbo","protocol":999999},"description":{"text":"A Limbo Server"}}"#
            .encode(w)
    }
}

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

                        let _ = connection.receive_packet::<ClientHandshakePacket>().await
                            .map_err(|err| error!("error while receiving handshake: {err}"));
                        let _ = connection.receive_packet::<ClientStatusPacket>().await
                            .map_err(|err| error!("error while receiving status request: {err}"));
                        let _ = connection.send_packet(TestResponse).await
                            .map_err(|err| error!("error while sending status response: {err}"));
                    },
                    Err(err) => error!("failed to accept connection: {err}"),
                }
            }
            _ = signal::ctrl_c() => break
        }
    }

    info!("shutting down");
}
