use std::{borrow::Cow, fmt::Display, sync::Arc};

use anyhow::anyhow;
use log::{debug, error, info};
use protocol::{
    chat::Message,
    info::{PlayerInfo, ServerInfo, VERSION},
    io::Raw,
    packets::{
        client::{
            handshake::ClientHandshakePacket, login::ClientLoginPacket, status::ClientStatusPacket,
            ClientPacket,
        },
        server::{
            login::ServerLoginPacket, play::ServerPlayPacket, status::ServerStatusPacket,
            ServerPacket,
        },
        State,
    },
    types::GameMode,
    ProtocolError, VarInt,
};
use tokio::{
    select,
    sync::{mpsc::Sender, RwLock},
};
use uuid::Uuid;

use crate::{config::Config, connection::Connection, shutdown::Shutdown, ServerError};

pub struct Client {
    config: Arc<RwLock<Config>>,

    shutdown: Shutdown,
    _shutdown_done: Sender<()>,

    connection: Connection,
    disconnected: bool,

    name: Option<String>,
    uuid: Option<Uuid>,
}

impl Client {
    pub fn new(
        connection: Connection,
        config: Arc<RwLock<Config>>,
        shutdown: Shutdown,
        shutdown_done: Sender<()>,
    ) -> Client {
        Client {
            config,

            shutdown,
            _shutdown_done: shutdown_done,

            connection,
            disconnected: false,

            name: None,
            uuid: None,
        }
    }

    pub async fn run(&mut self) {
        while !self.disconnected {
            select! {
                packet = self.connection.read_packet() => {
                    match packet {
                        Ok(Some(packet)) => {
                            if let Err(err) = self.process_packet(packet).await {
                                error!("failed to process packet: {:#}", anyhow!(err));
                            }
                        },
                        Ok(None) => self.disconnected = true,
                        Err(ServerError::Protocol(ProtocolError::InvalidPacketId(id))) => {
                            debug!(
                                "received unrecognized packet (state: {:?}, id: {:#04x})",
                                self.connection.state, id
                            );
                        },
                        Err(err) => error!("failed to read packet: {:#}", anyhow!(err)),
                    }
                }
                _ = self.shutdown.recv() => {
                    if let Err(err) = self.disconnect("Server is shutting down.").await {
                        error!("failed to disconnect client: {:#}", err);
                    }
                }
            }
        }

        if let State::Play = self.connection.state {
            info!(
                "client disconnected ({}, {})",
                self.name.as_ref().unwrap(),
                self.uuid.as_ref().unwrap()
            );
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
                    let config = self.config.read().await;
                    let response = ServerPacket::Status(ServerStatusPacket::Response {
                        response: ServerInfo::new(
                            VERSION,
                            PlayerInfo::simple(12, config.info.max_players),
                            Message::new(config.info.motd.clone()),
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
                    if name.is_empty() || name.len() > 16 {
                        return self
                            .disconnect("Usernames should be between 1-16 characters long.")
                            .await;
                    }

                    self.name = Some(name);
                    self.uuid = Some(Uuid::new_v4());

                    // TODO: Encryption
                    // TODO: Compression

                    self.connection
                        .write_packet(ServerPacket::Login(ServerLoginPacket::Success {
                            uuid: self.uuid.unwrap(),
                            name: self.name.clone().unwrap(),
                        }))
                        .await?;
                    self.connection.state = State::Play;

                    info!(
                        "client logged in ({}, {})",
                        self.name.as_ref().unwrap(),
                        self.uuid.as_ref().unwrap()
                    );

                    self.connection
                        .write_packet(ServerPacket::Play(ServerPlayPacket::JoinGame {
                            entity_id: 0,
                            hardcore: true,
                            gamemode: GameMode::Survival,
                            previous_gamemode: None,
                            world_names: vec!["limbo".to_string()],
                            dimension_codec: Raw(Cow::Borrowed(include_bytes!(
                                "./dimension_codec.nbt"
                            ))),
                            dimension: Raw(Cow::Borrowed(include_bytes!("./dimension.nbt"))),
                            world_name: "limbo".to_string(),
                            hashed_seed: 0,
                            max_players: VarInt(1),
                            view_distance: VarInt(32),
                            simulation_distance: VarInt(32),
                            reduced_debug_info: true,
                            enable_respawn_screen: false,
                            debug: false,
                            flat: false,
                        }))
                        .await?;
                }
            },
            ClientPacket::Play(_) => todo!(),
        }

        Ok(())
    }

    async fn disconnect<S: Display + ToString>(&mut self, reason: S) -> Result<(), ServerError> {
        self.disconnected = true;

        match self.connection.state {
            State::Login => {
                let disconnect = ServerPacket::Login(ServerLoginPacket::Disconnect {
                    reason: Message::new(reason.to_string()),
                });
                self.connection.write_packet(disconnect).await?;

                if let Some(name) = self.name.as_ref() {
                    info!("disallowed login for {} (reason: {})", name, reason);
                } else {
                    info!("disallowed login (reason: {})", reason);
                }
            }
            State::Play => {
                let disconnect = ServerPacket::Play(ServerPlayPacket::Disconnect {
                    reason: Message::new(reason.to_string()),
                });
                self.connection.write_packet(disconnect).await?;

                if let Some(name) = self.name.as_ref() {
                    info!("disconnected {} (reason: {})", name, reason);
                }
            }
            _ => {}
        }

        Ok(())
    }
}
