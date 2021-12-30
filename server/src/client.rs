use anyhow::anyhow;
use log::{error, warn};
use protocol::{
    info::{Motd, PlayerInfo, ServerInfo, VERSION},
    packets::{
        client::{handshake::ClientHandshakePacket, status::ClientStatusPacket, ClientPacket},
        server::{status::ServerStatusPacket, ServerPacket},
    },
    ProtocolError,
};

use crate::{connection::Connection, ServerError};

pub struct Client {
    connection: Connection,
    disconnected: bool,
}

impl Client {
    pub fn new(connection: Connection) -> Client {
        Client {
            connection,
            disconnected: false,
        }
    }

    pub async fn run(&mut self) {
        while !self.disconnected {
            match self.connection.read_packet().await {
                Ok(Some(packet)) => {
                    if let Err(err) = self.process_packet(packet).await {
                        error!("{:#}", anyhow!(err));
                        self.disconnected = true;
                    }
                }
                Ok(None) => self.disconnected = true,
                Err(ServerError::Protocol(ProtocolError::InvalidPacketId(id))) => {
                    warn!(
                        "received unrecognized packet ({:?}, id: {:#x})",
                        self.connection.state, id
                    );
                }
                Err(err) => {
                    error!("{:#}", anyhow!(err));
                    self.disconnected = true;
                }
            }
        }
    }

    async fn process_packet(&mut self, packet: ClientPacket) -> Result<(), ServerError> {
        match packet {
            ClientPacket::Handshake(packet) => match packet {
                ClientHandshakePacket::Handshake { next_state, .. } => {
                    // TODO: Check version if next state is login.
                    self.connection.state = next_state
                }
            },
            ClientPacket::Status(packet) => match packet {
                ClientStatusPacket::Request {} => {
                    let response = ServerPacket::Status(ServerStatusPacket::Response {
                        response: ServerInfo::new(
                            VERSION,
                            PlayerInfo::simple(12, -1),
                            Motd::new("Limbo".into()),
                        ),
                    });

                    self.connection.write_packet(response).await?;
                }
                ClientStatusPacket::Ping { payload } => {
                    let pong = ServerPacket::Status(ServerStatusPacket::Pong { payload });
                    self.connection.write_packet(pong).await?;
                }
            },
        }

        Ok(())
    }
}
