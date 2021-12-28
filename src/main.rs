use std::{
    io::{Cursor, Read},
    net::TcpListener,
};

use anyhow::{Context, Result};
use protocol::{Readable, VarInt};

use crate::protocol::packets::client::handshake::ClientHandshakePacket;

mod protocol;

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565").context("failed to open TCP listener")?;

    for stream in listener.incoming() {
        let mut stream = stream?;

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
        println!(
            "{}: {:?}",
            packet_id,
            ClientHandshakePacket::read_from(&mut cursor)
        );
    }

    Ok(())
}
