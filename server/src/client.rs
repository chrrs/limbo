use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::anyhow;
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use protocol::{
    chat::Message,
    info::{ServerInfo, ServerPlayerInfo, VERSION},
    io::{BooleanPrefixedOption, RawBytes, VarIntPrefixedVec},
    metadata::{EntityMetadata, MetaIndex, MetaType},
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
    player_info::{AddPlayerAction, AddPlayerProperty, PlayerInfo},
    types::{GameMode, Position},
    PacketField, ReadError, VarInt,
};
use rand::{rngs::OsRng, Rng};
use rsa::{PaddingScheme, PublicKeyParts, RsaPrivateKey};
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
    mojang::{self, AuthenticationResponse},
    shutdown::Shutdown,
};

const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(5);

// TODO: Find a better place to store this variable.
pub static ONLINE_PLAYERS: AtomicUsize = AtomicUsize::new(0);

static UNIVERSAL_RSA: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::new(&mut OsRng, 1024).expect("failed to generate server RSA private key")
});

static UNIVERSAL_ENCODED_RSA_PUBLIC_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
    rsa_der::public_key_to_der(
        &UNIVERSAL_RSA.n().to_bytes_be(),
        &UNIVERSAL_RSA.e().to_bytes_be(),
    )
});

static UNIVERSAL_VERIFY_TOKEN: Lazy<Vec<u8>> = Lazy::new(|| (0..4).map(|_| OsRng.gen()).collect());

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
                            error!("failed to read packet: {:#}", anyhow!(err));
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
            info!("client disconnected ({}, {})", self.name(), self.uuid());

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
                    let mut config = self.config.write().await;

                    let player_info = if config.info.hide_player_count {
                        None
                    } else {
                        Some(ServerPlayerInfo::simple(
                            ONLINE_PLAYERS.load(Ordering::Relaxed) as isize,
                            config.info.max_players,
                        ))
                    };

                    let response = ServerPacket::Status(ServerStatusPacket::Response {
                        response: ServerInfo::new(
                            VERSION,
                            player_info,
                            Message::new(config.info.motd.clone()),
                            config.info.icon(),
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
                        self.disconnect("Usernames should be between 1-16 characters long.")
                            .await?;
                        return Ok(());
                    }

                    self.name = Some(name);

                    self.connection
                        .write_packet(ServerPacket::Login(ServerLoginPacket::EncryptionRequest {
                            server_id: String::new(),
                            public_key: VarIntPrefixedVec(UNIVERSAL_ENCODED_RSA_PUBLIC_KEY.clone()),
                            verify_token: VarIntPrefixedVec(UNIVERSAL_VERIFY_TOKEN.clone()),
                        }))
                        .await?;
                }
                ClientLoginPacket::EncryptionResponse {
                    shared_secret,
                    verify_token,
                } => {
                    let verify_token =
                        UNIVERSAL_RSA.decrypt(PaddingScheme::PKCS1v15Encrypt, &verify_token.0);
                    let shared_secret =
                        UNIVERSAL_RSA.decrypt(PaddingScheme::PKCS1v15Encrypt, &shared_secret.0);

                    let shared_secret = match (verify_token, shared_secret) {
                        (Ok(verify_token), Ok(shared_secret))
                            if verify_token == UNIVERSAL_VERIFY_TOKEN[..] =>
                        {
                            shared_secret
                        }
                        _ => {
                            self.disconnect("Invalid encryption challenge response.")
                                .await?;
                            return Ok(());
                        }
                    };

                    if self.connection.update_encryption(&shared_secret).is_err() {
                        self.disconnect("Unexpected shared secret key length.")
                            .await?;
                        return Ok(());
                    }

                    let config_clone = self.config.clone();
                    let config = config_clone.read().await;

                    self.set_compression(256).await?;

                    let response = match mojang::authenticate(
                        "",
                        &shared_secret,
                        &UNIVERSAL_ENCODED_RSA_PUBLIC_KEY,
                        self.name(),
                    ) {
                        Ok(response) => response,
                        Err(err) => {
                            error!("failed to authenticate {}: {:#}", self.name(), anyhow!(err));
                            self.disconnect("Could not validate session.").await?;
                            return Ok(());
                        }
                    };

                    let AuthenticationResponse { id, properties } = response;
                    self.uuid = Some(id);

                    self.connection
                        .write_packet(ServerPacket::Login(ServerLoginPacket::Success {
                            uuid: self.uuid.unwrap(),
                            name: self.name.clone().unwrap(),
                        }))
                        .await?;
                    self.connection.state = State::Play;

                    info!("client logged in ({}, {})", self.name(), self.uuid());

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

                    // TODO: Abstract this away in some kind of TAB-screen handler.
                    self.connection
                        .write_packet(ServerPacket::Play(ServerPlayPacket::PlayerInfo {
                            info: PlayerInfo::AddPlayer(VarIntPrefixedVec(vec![AddPlayerAction {
                                uuid: *self.uuid(),
                                name: self.name().to_string(),
                                properties: VarIntPrefixedVec(
                                    properties
                                        .into_iter()
                                        .map(|p| AddPlayerProperty {
                                            name: p.name,
                                            value: p.value,
                                            signature: BooleanPrefixedOption(p.signature),
                                        })
                                        .collect(),
                                ),
                                game_mode: GameMode::Survival,
                                ping: VarInt(0), // TODO: Appropriately set this.
                                display_name: BooleanPrefixedOption(None),
                            }])),
                        }))
                        .await?;
                }
            },
            ClientPacket::Play(packet) => match packet {
                ClientPlayPacket::TeleportConfirm { .. } => {}
                ClientPlayPacket::PluginMessage { channel, data } => match channel.as_str() {
                    "minecraft:brand" => match String::read_from(&mut &data.0[..]) {
                        Ok(brand) => debug!("client brand of {} is {}", self.name(), brand),
                        Err(err) => warn!(
                            "failed to process client brand of {}: {:#}",
                            self.name(),
                            anyhow!(err)
                        ),
                    },
                    _ => debug!(
                        "received unknown plugin message (channel: {}, from: {})",
                        channel,
                        self.name()
                    ),
                },
                ClientPlayPacket::PlayerPosition { .. } => {}
                ClientPlayPacket::ClientSettings {
                    displayed_skin_parts,
                    main_hand,
                    ..
                } => {
                    self.connection
                        .write_packet(ServerPacket::Play(ServerPlayPacket::EntityMetadata {
                            // TODO: Don't hardcode the entity ID.
                            id: VarInt(0),
                            metadata: EntityMetadata::new()
                                .with(MetaIndex::SkinParts, MetaType::Byte(displayed_skin_parts))
                                .with(MetaIndex::MainHand, MetaType::Byte(main_hand.0 as u8)),
                        }))
                        .await?;
                }
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
            self.name()
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

    pub fn name(&self) -> &str {
        self.name.as_ref().unwrap()
    }

    pub fn uuid(&self) -> &Uuid {
        self.uuid.as_ref().unwrap()
    }
}
