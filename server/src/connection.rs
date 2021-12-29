use std::io::Cursor;

use bytes::{Buf, BytesMut};
use protocol::{
    packets::{client::ClientPacket, server::ServerPacket, State},
    Readable, VarInt, Writable,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    packet_buf: Vec<u8>,
    buffer: BytesMut,
    pub state: State,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            packet_buf: Vec::new(),
            buffer: BytesMut::new(),
            state: State::Handshake,
        }
    }

    pub async fn read_packet(&mut self) -> anyhow::Result<Option<ClientPacket>> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                return Ok(Some(packet));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    // TODO: Properly return an error here.
                    return Err(anyhow::anyhow!("connection reset by peer"));
                }
            }
        }
    }

    pub fn parse_packet(&mut self) -> anyhow::Result<Option<ClientPacket>> {
        let (offset, length) = {
            let mut buf = Cursor::new(&self.buffer[..]);
            if let Ok(length) = VarInt::read_from(&mut buf) {
                if self.buffer.len() < length.0 as usize + buf.position() as usize {
                    return Ok(None);
                }

                (buf.position() as usize, length.0 as usize)
            } else {
                return Ok(None);
            }
        };

        self.buffer.advance(offset);
        let mut buf = Cursor::new(&self.buffer[..length]);
        let packet = ClientPacket::decode(self.state, &mut buf)?;
        self.buffer.advance(length);

        Ok(Some(packet))
    }

    pub async fn write_packet(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        packet.encode_to(&mut self.packet_buf)?;
        let length = self.packet_buf.len();
        VarInt(length as i32).write_to(&mut self.packet_buf)?;

        self.stream.write_all(&self.packet_buf[length..]).await?;
        self.stream.write_all(&self.packet_buf[..length]).await?;

        self.packet_buf.clear();

        self.stream.flush().await?;

        Ok(())
    }
}
