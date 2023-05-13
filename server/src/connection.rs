use std::io::{Cursor, Read, Write};

use aes::{
    cipher::{errors::InvalidLength, AsyncStreamCipher, NewCipher},
    Aes128,
};
use bytes::{Buf, BytesMut};
use cfb8::Cfb8;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use log::trace;
use protocol::{
    packets::{client::ClientPacket, server::ServerPacket, State},
    PacketField, VarInt,
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error("connection closed")]
    ConnectionClosed,

    #[error(
        "invalid reported decompressed packet length (reported: {reported}, actual: {actual})"
    )]
    InvalidReportedLength { reported: usize, actual: usize },

    #[error("packet length decoding error")]
    PacketLengthEncode(#[from] protocol::FieldReadError),

    #[error("failed to decode packet")]
    Decode(#[from] protocol::ReadError),

    #[error("failed to read from stream")]
    Read(#[from] std::io::Error),

    #[error("failed to decompress received data")]
    Decompression(#[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("failed to encode packet length")]
    PacketLengthEncode(#[from] protocol::FieldWriteError),

    #[error("failed to encode packet")]
    Encode(#[from] protocol::WriteError),

    #[error("failed to write to stream")]
    Write(#[from] std::io::Error),

    #[error("failed to compress packet")]
    Compression(#[source] std::io::Error),
}

pub struct Connection {
    stream: BufWriter<TcpStream>,
    packet_buf: Vec<u8>,
    compression_buf: Vec<u8>,
    staging_buf: Vec<u8>,
    buffer: BytesMut,
    pub state: State,
    pub compression_threshold: Option<usize>,
    cipher: Option<(Cfb8<Aes128>, Cfb8<Aes128>)>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            packet_buf: Vec::new(),
            compression_buf: Vec::new(),
            staging_buf: Vec::new(),
            buffer: BytesMut::new(),
            state: State::Handshake,
            compression_threshold: None,
            cipher: None,
        }
    }

    pub fn update_encryption(&mut self, shared_secret: &[u8]) -> Result<(), InvalidLength> {
        self.cipher = Some((
            Cfb8::new_from_slices(shared_secret, shared_secret)?,
            Cfb8::new_from_slices(shared_secret, shared_secret)?,
        ));

        trace!("encrypted connection");

        Ok(())
    }

    pub async fn read_packet(&mut self) -> Result<Option<ClientPacket>, ReceiveError> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                return Ok(Some(packet));
            }

            let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
            if bytes_read == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ReceiveError::ConnectionClosed);
                }
            } else if let Some(cipher) = &mut self.cipher {
                let i = self.buffer.len() - bytes_read;
                cipher.0.decrypt(&mut self.buffer[i..]);
            }
        }
    }

    pub fn parse_packet(&mut self) -> Result<Option<ClientPacket>, ReceiveError> {
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
                decoder
                    .read_to_end(&mut self.packet_buf)
                    .map_err(ReceiveError::Decompression)?;

                if data_length as usize != self.packet_buf.len() {
                    return Err(ReceiveError::InvalidReportedLength {
                        reported: data_length as usize,
                        actual: self.packet_buf.len(),
                    });
                }

                let packet = ClientPacket::decode(self.state, &mut &self.packet_buf[..]);
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

    pub async fn write_packet(&mut self, packet: ServerPacket) -> Result<(), SendError> {
        packet.encode_to(&mut self.packet_buf)?;

        if let Some(threshold) = self.compression_threshold {
            if threshold < self.packet_buf.len() {
                stage_compressed_packet_into(
                    &mut self.compression_buf,
                    &mut self.staging_buf,
                    &self.packet_buf,
                )?;
            } else {
                stage_packet_into(&mut self.staging_buf, &self.packet_buf)?;
            }
        } else {
            stage_packet_into(&mut self.staging_buf, &self.packet_buf)?;
        }

        if let Some(cipher) = &mut self.cipher {
            cipher.1.encrypt(&mut self.staging_buf);
        }

        self.stream.write_all(&self.staging_buf).await?;
        self.stream.flush().await?;
        self.staging_buf.clear();

        trace!("sent packet: {:?}", packet);

        self.packet_buf.clear();

        Ok(())
    }
}

// TODO: Ideally, these should be struct methods, but the borrow checker doesn't like that.
fn stage_compressed_packet_into(
    mut compression_buf: &mut Vec<u8>,
    staging_buf: &mut Vec<u8>,
    packet_buf: &[u8],
) -> Result<(), SendError> {
    let length = packet_buf.len();

    trace!("compressing packet of {} bytes", length);
    VarInt(length as i32).write_to(compression_buf)?;
    let mut encoder = ZlibEncoder::new(&mut compression_buf, Compression::default());
    encoder
        .write_all(packet_buf)
        .map_err(SendError::Compression)?;
    encoder.finish().map_err(SendError::Compression)?;

    stage_packet_into(staging_buf, compression_buf)?;
    compression_buf.clear();

    Ok(())
}

fn stage_packet_into(staging_buf: &mut Vec<u8>, packet_buf: &[u8]) -> Result<(), SendError> {
    let mut buf = [0u8; 5];
    let mut length_bytes = Cursor::new(&mut buf[..]);
    VarInt(packet_buf.len() as i32).write_to(&mut length_bytes)?;
    let position = length_bytes.position() as usize;

    staging_buf.extend_from_slice(&buf[..position]);
    staging_buf.extend_from_slice(packet_buf);

    Ok(())
}
