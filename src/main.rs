use std::{
    io::{Cursor, Read, Write},
    net::TcpListener,
};

use anyhow::{Context, Result};
use protocol::{Readable, VarInt};

use crate::protocol::{
    packets::{client::handshake::ClientHandshakePacket, server::status::ServerResponsePacket},
    Writable,
};

mod protocol;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565").context("failed to open TCP listener")?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        let mut status = false;
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

            let packet_id = VarInt::read_from(&mut cursor).context("failed to read packet id")?;

            match packet_id.0 {
                0 if status => {
                    println!("request packet received");

                    let mut buffer = Vec::new();
                    let packet = ServerResponsePacket {
                        response: r#"
                        {
                            "version": {
                                "name": "1.18.1",
                                "protocol": 757
                            },
                            "players": {
                                "max": 10,
                                "online": 12,
                                "sample": [
                                    {
                                        "name": "thinkofdeath",
                                        "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"
                                    }
                                ]
                            },
                            "description": {
                                "text": "Hello world"
                            }
                        }
                        "#
                        .to_string(),
                    };

                    VarInt(0).write_to(&mut buffer)?;
                    packet.write_to(&mut buffer)?;

                    let mut result = Vec::new();
                    VarInt(buffer.len() as i32).write_to(&mut result)?;
                    result.append(&mut buffer);

                    stream.write_all(&result)?;
                }
                0 => {
                    let packet = ClientHandshakePacket::read_from(&mut cursor)
                        .context("failed to read handshake packet")?;
                    println!("{:?}", packet);
                    status = packet.next_state.0 == 1;
                }
                _ => println!("unrecognized packet with id {}", packet_id),
            }
        }
    }

    Ok(())
}
