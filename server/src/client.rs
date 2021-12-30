use anyhow::anyhow;
use log::{error, info, trace, warn};
use protocol::{
    info::{Motd, PlayerInfo, ServerInfo, VERSION},
    packets::{
        client::{
            handshake::ClientHandshakePacket, login::ClientLoginPacket, status::ClientStatusPacket,
            ClientPacket,
        },
        server::{login::ServerLoginPacket, status::ServerStatusPacket, ServerPacket},
        State,
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
                        let err = anyhow!(err);
                        error!("failed to process packet: {:#}", err);

                        if let Err(err) = self.disconnect(&format!("Bad packet: {}", err)).await {
                            error!(
                                "failed to disconnect client after packet error: {:#}",
                                anyhow!(err)
                            )
                        }
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
                    let err = anyhow!(err);
                    error!("failed to read packet: {:#}", err);

                    if let Err(err) = self.disconnect(&format!("Invalid packet: {}", err)).await {
                        error!(
                            "failed to disconnect client after packet error: {:#}",
                            anyhow!(err)
                        )
                    }
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
            ClientPacket::Login(packet) => match packet {
                ClientLoginPacket::Start { name } => {
                    trace!("client logged in with name {}", name);

                    self.disconnect(&format!("Your name is {}", name)).await?;
                }
            },
        }

        Ok(())
    }

    async fn disconnect(&mut self, reason: &str) -> Result<(), ServerError> {
        self.disconnected = true;

        match self.connection.state {
            State::Login => {
                let disconnect = ServerPacket::Login(ServerLoginPacket::Disconnect {
                    // TODO: This should use some global chat message wrapper
                    reason: format!("{{ \"text\":\"{}\" }}", reason),
                });
                self.connection.write_packet(disconnect).await?;

                // TODO: Include some user specific information in here.
                info!("disallowed login with reason: {}", reason);
            }
            State::Play => todo!(),
            _ => {}
        }

        Ok(())
    }
}
