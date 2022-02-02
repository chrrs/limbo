use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::anyhow;
use log::{debug, error, info, trace, warn};
use protocol::{
    chat::Message,
    info::{PlayerInfo, ServerInfo, VERSION},
    io::{RawBytes, VarIntPrefixedVec},
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
    types::{GameMode, Position},
    PacketField, ReadError, VarInt,
};
use thiserror::Error;
use tokio::{
    select,
    sync::{broadcast, mpsc, RwLock},
    time,
};
use uuid::Uuid;

use crate::{
    config::Config,
    connection::{Connection, ReceiveError, SendError},
    shutdown::Shutdown,
};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(5);

// TODO: Find a better place to store this variable.
pub static ONLINE_PLAYERS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Error)]
pub enum PacketProcessingError {
    #[error("failed to send response packet")]
    Send(#[from] SendError),
}

pub struct Client {
    config: Arc<RwLock<Config>>,

    shutdown: Shutdown,
    _shutdown_done: mpsc::Sender<()>,
    stop: broadcast::Sender<()>,

    connection: Connection,
    disconnected: bool,

    packet_send: mpsc::Sender<ServerPacket>,
    packet_queue: mpsc::Receiver<ServerPacket>,

    name: Option<String>,
    uuid: Option<Uuid>,
}

impl Client {
    pub fn new(
        connection: Connection,
        config: Arc<RwLock<Config>>,
        shutdown: Shutdown,
        shutdown_done: mpsc::Sender<()>,
    ) -> Client {
        let (sender, receiver) = mpsc::channel(5);

        Client {
            config,

            shutdown,
            _shutdown_done: shutdown_done,

            connection,
            disconnected: false,

            packet_send: sender,
            packet_queue: receiver,
            stop: broadcast::channel(1).0,

            name: None,
            uuid: None,
        }
    }

    pub async fn run(&mut self) {
        while !self.disconnected {
            select! {
                Some(packet) = self.packet_queue.recv() => {
                    if let Err(err) = self.connection.write_packet(packet).await {
                        error!("failed to write packet: {:#}", anyhow!(err));
                    }
                }
                packet = self.connection.read_packet() => {
                    match packet {
                        Ok(Some(packet)) => {
                            if let Err(err) = self.process_packet(packet).await {
                                error!("failed to process packet: {:#}", anyhow!(err));
                            }
                        },
                        Ok(None) => self.disconnected = true,
                        Err(ReceiveError::Decode(ReadError::UnrecognizedPacketId(id))) => {
                            debug!(
                                "received unrecognized packet (state: {:?}, id: {:#04x})",
                                self.connection.state, id
                            );
                        },
                        Err(ReceiveError::ConnectionClosed) => self.disconnected = true,
                        Err(err) => {
                            warn!("failed to read packet: {:#}", anyhow!(err));
                            let _ = self.disconnect("Bad packet.").await;
                        }
                    }
                }
                _ = self.shutdown.recv() => {
                    if let Err(err) = self.disconnect("Server is shutting down.").await {
                        error!("failed to disconnect client: {:#}", err);
                    }
                }
            }
        }

        // This will fail if there are no recipients, but we don't care.
        let _ = self.stop.send(());

        if let State::Play = self.connection.state {
            info!(
                "client disconnected ({}, {})",
                self.name.as_ref().unwrap(),
                self.uuid.as_ref().unwrap()
            );

            ONLINE_PLAYERS.fetch_sub(1, Ordering::Relaxed);
        }
    }

    async fn process_packet(&mut self, packet: ClientPacket) -> Result<(), PacketProcessingError> {
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
                        Some(PlayerInfo::simple(
                            ONLINE_PLAYERS.load(Ordering::Relaxed) as isize,
                            config.info.max_players,
                        ))
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
                    let config_clone = self.config.clone();
                    let config = config_clone.read().await;

                    if name.is_empty() || name.len() > 16 {
                        self.disconnect("Usernames should be between 1-16 characters long.")
                            .await?;
                        return Ok(());
                    }

                    self.name = Some(name);
                    self.uuid = Some(Uuid::new_v4());

                    // TODO: Encryption

                    // TODO: Lower compression threshold.
                    self.set_compression(256).await?;

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

                    ONLINE_PLAYERS.fetch_add(1, Ordering::Relaxed);

                    self.start_keeping_alive();

                    self.connection
                        .write_packet(ServerPacket::Play(ServerPlayPacket::JoinGame {
                            entity_id: 0,
                            hardcore: true,
                            gamemode: GameMode::Survival,
                            previous_gamemode: None,
                            world_names: VarIntPrefixedVec(vec!["limbo".to_string()]),
                            dimension_codec: RawBytes::new(
                                &include_bytes!("./dimension_codec.nbt")[..],
                            ),
                            dimension: RawBytes::new(&include_bytes!("./dimension.nbt")[..]),
                            world_name: "limbo".to_string(),
                            hashed_seed: 0,
                            max_players: VarInt(1),
                            view_distance: VarInt(32),
                            simulation_distance: VarInt(32),
                            reduced_debug_info: false,
                            enable_respawn_screen: false,
                            debug: false,
                            flat: false,
                        }))
                        .await?;

                    self.send_plugin_message("minecraft:brand", &config.info.name)
                        .await?;

                    self.connection
                        .write_packet(ServerPacket::Play(ServerPlayPacket::SpawnPosition {
                            angle: 0.0,
                            location: Position::new(0, 64, 0),
                        }))
                        .await?;

                    // TODO: Abstract this away into a proper teleport function.
                    self.connection
                        .write_packet(ServerPacket::Play(
                            ServerPlayPacket::PlayerPositionAndLook {
                                x: 0.0,
                                y: 64.0,
                                z: 0.0,
                                yaw: 0.0,
                                pitch: 0.0,
                                flags: 0,
                                teleport_id: VarInt(0),
                                dismount_vehicle: true,
                            },
                        ))
                        .await?;
                }
            },
            ClientPacket::Play(packet) => match packet {
                ClientPlayPacket::PluginMessage { channel, data } => match channel.as_str() {
                    "minecraft:brand" => {
                        let brand = String::read_from_slice(&data.0);
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
                ClientPlayPacket::PlayerPosition { x, y, z, on_ground } => {
                    trace!(
                        "{} moved to {:.02}, {:.02}, {:.02} (grounded: {})",
                        self.name.as_ref().unwrap(),
                        x,
                        y,
                        z,
                        on_ground
                    );
                }
                ClientPlayPacket::ClientSettings { .. } => {}
                ClientPlayPacket::KeepAlive { .. } => {
                    // TODO: Check if the ID matches.
                }
                ClientPlayPacket::PlayerPositionAndRotation { .. } => {}
            },
        }

        Ok(())
    }

    async fn set_compression(&mut self, threshold: usize) -> Result<(), SendError> {
        self.connection
            .write_packet(ServerPacket::Login(ServerLoginPacket::SetCompression {
                threshold: VarInt(threshold as i32),
            }))
            .await?;

        self.connection.compression_threshold = Some(threshold);

        Ok(())
    }

    fn start_keeping_alive(&mut self) {
        let send_queue = self.packet_send.clone();
        let mut stop = self.stop.subscribe();
        tokio::spawn(async move {
            let mut interval = time::interval(KEEP_ALIVE_INTERVAL);

            loop {
                select! {
                    _ = interval.tick() => {
                        // TODO: Properly process the client response to this.
                        send_queue.send(ServerPacket::Play(ServerPlayPacket::KeepAlive { id: 0 }))
                            .await
                            .unwrap();
                    }
                    _ = stop.recv() => break
                }
            }

            debug!("halting keep-alives");
        });
    }

    async fn send_plugin_message<S: Display + ToString, D: PacketField>(
        &mut self,
        channel: S,
        data: &D,
    ) -> Result<(), SendError> {
        self.connection
            .write_packet(ServerPacket::Play(ServerPlayPacket::PluginMessage {
                channel: channel.to_string(),
                data: RawBytes::new(data.write_to_vec()?),
            }))
            .await?;

        debug!(
            "sent plugin message (channel: {}, to: {})",
            channel,
            self.name.as_ref().unwrap()
        );

        Ok(())
    }

    async fn disconnect<S: Display + ToString>(&mut self, reason: S) -> Result<(), SendError> {
        // TODO: Actually disconnect when this function is called, instead of after the next packet.

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
