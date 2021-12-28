use std::{
    io::{Cursor, Read, Write},
    net::TcpListener,
};

use anyhow::{Context, Result};
use protocol::{Readable, VarInt};

use crate::protocol::{
    info::{Motd, PlayerInfo, ServerInfo, VERSION},
    packets::{
        client::{handshake::ClientHandshakePacket, status::ClientStatusPacket, ClientPacket},
        server::{status::ServerStatusPacket, ServerPacket},
        State,
    },
    Writable,
};

mod protocol;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565").context("failed to open TCP listener")?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut state = State::Handshake;
        loop {
            let mut buffer = [0; 5];
            stream
                .peek(&mut buffer)
                .context("failed to peek for packet length")?;
            let mut length_cursor = Cursor::new(&buffer[..]);
            let length = VarInt::read_from(&mut length_cursor)
                .context("failed to read packet length")?
                .0 as usize
                + length_cursor.position() as usize;

            let mut buffer = vec![0; length];
            stream
                .read_exact(&mut buffer)
                .context("failed to read packet into buffer")?;
            let mut cursor = Cursor::new(&buffer[..]);
            cursor.set_position(length_cursor.position());

            let packet = ClientPacket::decode(state, &mut cursor)
                .context("failed to decode received packet")?;
            println!("received {:?}", packet);

            match packet {
                ClientPacket::Handshake(ClientHandshakePacket::Handshake {
                    next_state, ..
                }) => state = next_state,
                ClientPacket::Status(ClientStatusPacket::Request {}) => {
                    let mut payload = Vec::new();

                    let packet = ServerPacket::Status(ServerStatusPacket::Response {
                        response: serde_json::to_string(&ServerInfo::new(
                            VERSION,
                            PlayerInfo::simple(10, 10),
                            Motd::new("Limbo".into()),
                        ))
                        .context("failed to serialize ping packet")?,
                    });

                    let mut data = packet
                        .encode()
                        .context("failed to encode ping response packet")?;
                    VarInt(data.len() as i32)
                        .write_to(&mut payload)
                        .context("failed to write packet length")?;
                    payload.append(&mut data);

                    stream
                        .write_all(&payload)
                        .context("failed to send packet")?;

                    println!("sent {:?}", packet);
                }
            }
        }
    }

    Ok(())
}
