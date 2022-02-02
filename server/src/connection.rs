use std::io::{Cursor, Read, Write};

use bytes::{Buf, BytesMut};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use log::trace;
use protocol::{
    packets::{client::ClientPacket, server::ServerPacket, State},
    Readable, VarInt, Writable,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::ServerError;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    packet_buf: Vec<u8>,
    compression_buf: Vec<u8>,
    buffer: BytesMut,
    pub state: State,
    pub compression_threshold: Option<usize>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            packet_buf: Vec::new(),
            compression_buf: Vec::new(),
            buffer: BytesMut::new(),
            state: State::Handshake,
            compression_threshold: None,
        }
    }

    pub async fn read_packet(&mut self) -> Result<Option<ClientPacket>, ServerError> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                return Ok(Some(packet));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ServerError::ConnectionReset);
                }
            }
        }
    }

    pub fn parse_packet(&mut self) -> Result<Option<ClientPacket>, ServerError> {
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

        let packet = if self.compression_threshold.is_some() {
            let data_length = VarInt::read_from(&mut buf)?.0;
            if data_length == 0 {
                ClientPacket::decode(self.state, &mut buf)
            } else {
                trace!("decompressing packet of {} bytes", data_length);
                let mut decoder = ZlibDecoder::new(&mut buf);
                decoder.read_to_end(&mut self.packet_buf)?;
                // TODO: Check length.
                let packet = ClientPacket::decode(self.state, &mut Cursor::new(&self.packet_buf));
                self.packet_buf.clear();
                packet
            }
        } else {
            ClientPacket::decode(self.state, &mut buf)
        };

        self.buffer.advance(length);

        // We defer the propagation of the error to correctly ignore unrecognized packets.
        let packet = packet?;
        trace!("received packet: {:?}", packet);
        Ok(Some(packet))
    }

    pub async fn write_packet(&mut self, packet: ServerPacket) -> Result<(), ServerError> {
        packet.encode_to(&mut self.packet_buf)?;

        if let Some(threshold) = self.compression_threshold {
            self.write_compressed(threshold).await?;

            trace!("sent packet: {:?}", packet);
        } else {
            self.write_uncompressed().await?;

            trace!("sent packet (uncompressed): {:?}", packet);
        }

        self.packet_buf.clear();

        Ok(())
    }

    async fn write_compressed(&mut self, threshold: usize) -> Result<(), ServerError> {
        let length = self.packet_buf.len();

        if length < threshold {
            VarInt(0).write_to(&mut self.compression_buf)?;
            self.compression_buf.extend_from_slice(&self.packet_buf);
        } else {
            trace!("compressing packet of {} bytes", length);
            VarInt(length as i32).write_to(&mut self.compression_buf)?;
            let mut encoder = ZlibEncoder::new(&mut self.compression_buf, Compression::default());
            encoder.write_all(&self.packet_buf)?;
            encoder.finish()?;
        }

        let mut buf = [0u8; 5];
        let mut length_bytes = Cursor::new(&mut buf[..]);
        VarInt(self.compression_buf.len() as i32).write_to(&mut length_bytes)?;
        let position = length_bytes.position() as usize;

        self.stream.write_all(&buf[..position]).await?;
        self.stream.write_all(&self.compression_buf).await?;
        self.stream.flush().await?;

        self.compression_buf.clear();

        Ok(())
    }

    async fn write_uncompressed(&mut self) -> Result<(), ServerError> {
        let mut buf = [0u8; 5];
        let mut length_bytes = Cursor::new(&mut buf[..]);
        VarInt(self.packet_buf.len() as i32).write_to(&mut length_bytes)?;
        let position = length_bytes.position() as usize;

        self.stream.write_all(&buf[..position]).await?;
        self.stream.write_all(&self.packet_buf).await?;
        self.stream.flush().await?;

        Ok(())
    }
}
