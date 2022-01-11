use std::{borrow::Cow, fmt::Display, sync::Arc};

use anyhow::anyhow;
use log::{debug, error, info};
use protocol::{
    chat::Message,
    info::{PlayerInfo, ServerInfo, VERSION},
    io::Raw,
    packets::{
        client::{
            handshake::ClientHandshakePacket, login::ClientLoginPacket, play::ClientPlayPacket,
            status::ClientStatusPacket, ClientPacket,
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
                        Err(ServerError::ConnectionReset) => self.disconnected = true,
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
                    } else if self.config.read().await.info.hidden {
                        self.disconnect("").await?;
                    }
                }
            },
            ClientPacket::Status(packet) => match packet {
                ClientStatusPacket::Request {} => {
                    let config = self.config.read().await;

                    let player_info = if config.info.hide_player_count {
                        None
                    } else {
                        // TODO: Make this player count accurate.
                        Some(PlayerInfo::simple(12, config.info.max_players))
                    };

                    let response = ServerPacket::Status(ServerStatusPacket::Response {
                        response: ServerInfo::new(
                            VERSION,
                            player_info,
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
                            dimension_codec: Raw::new(&include_bytes!("./dimension_codec.nbt")[..]),
                            dimension: Raw::new(&include_bytes!("./dimension.nbt")[..]),
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

                    self.send_plugin_message("minecraft:brand", "limbo".as_bytes())
                        .await?;
                }
            },
            ClientPacket::Play(packet) => match packet {
                ClientPlayPacket::PluginMessage { channel, data } => match channel.as_str() {
                    "minecraft:brand" => {
                        let brand = std::str::from_utf8(&data.0);
                        if let Ok(brand) = brand {
                            debug!(
                                "client brand of {} is {}",
                                self.name.as_ref().unwrap(),
                                brand
                            )
                        }
                    }
                    _ => debug!(
                        "received unknown plugin message (channel: {}, from: {})",
                        channel,
                        self.name.as_ref().unwrap()
                    ),
                },
            },
        }

        Ok(())
    }

    async fn send_plugin_message<S: Display + ToString, D: Into<Cow<'static, [u8]>>>(
        &mut self,
        channel: S,
        data: D,
    ) -> Result<(), ServerError> {
        self.connection
            .write_packet(ServerPacket::Play(ServerPlayPacket::PluginMessage {
                channel: channel.to_string(),
                data: Raw::new(data),
            }))
            .await?;

        debug!(
            "sent plugin message (channel: {}, to: {})",
            channel,
            self.name.as_ref().unwrap()
        );

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
                    info!("disallowed login ({}, reason: {})", name, reason);
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
