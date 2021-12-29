use connection::Connection;
use protocol::{
    info::{Motd, PlayerInfo, ServerInfo, VERSION},
    packets::{
        client::{status::ClientStatusPacket, ClientPacket},
        server::{status::ServerStatusPacket, ServerPacket},
        State,
    },
};
use tokio::net::TcpListener;

mod connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut connection = Connection::new(stream);
            println!("- new connection made");
            println!("received {:?}", connection.read_packet().await);
            connection.state = State::Status;
            println!("received {:?}", connection.read_packet().await);

            {
                let packet = ServerPacket::Status(ServerStatusPacket::Response {
                    response: ServerInfo::new(
                        VERSION,
                        PlayerInfo::simple(12, -1),
                        Motd::new("Limbo".into()),
                    ),
                });
                println!("sent {:?}", packet);
                connection.write_packet(packet).await.unwrap();
            }

            let ping = connection.read_packet().await;
            println!("received {:?}", ping);
            if let Ok(Some(ClientPacket::Status(ClientStatusPacket::Ping { payload }))) = ping {
                let packet = ServerPacket::Status(ServerStatusPacket::Pong { payload });
                println!("sent {:?}", packet);
                connection.write_packet(packet).await.unwrap();
            }
        });
    }
}
