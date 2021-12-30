use anyhow::anyhow;
use log::{error, info, trace, warn};
use protocol::{
    chat::Message,
    info::{PlayerInfo, ServerInfo, VERSION},
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

    name: Option<String>,
}

impl Client {
    pub fn new(connection: Connection) -> Client {
        Client {
            connection,
            disconnected: false,

            name: None,
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
                ClientHandshakePacket::Handshake {
                    next_state,
                    protocol_version,
                    ..
                } => {
                    self.connection.state = next_state;

                    if let State::Login = next_state {
                        if VERSION.protocol != protocol_version.0 as usize {
                            self.disconnect(&format!("Version mismatch between client and server. Please connect using {}.", VERSION.name)).await?;
                        }
                    }
                }
            },
            ClientPacket::Status(packet) => match packet {
                ClientStatusPacket::Request {} => {
                    let response = ServerPacket::Status(ServerStatusPacket::Response {
                        response: ServerInfo::new(
                            VERSION,
                            PlayerInfo::simple(12, -1),
                            Message::new("Limbo"),
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
                    self.name = Some(name);

                    self.disconnect("Unimplemented").await?;
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
                    reason: Message::new(reason.to_string()),
                });
                self.connection.write_packet(disconnect).await?;

                if let Some(name) = self.name.as_ref() {
                    info!("disallowed login for {} with reason: {}", name, reason);
                } else {
                    info!("disallowed login with reason: {}", reason);
                }
            }
            State::Play => todo!(),
            _ => {}
        }

        Ok(())
    }
}
