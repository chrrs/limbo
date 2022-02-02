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

    #[error("decoding error")]
    Decode(#[from] protocol::ReadError),

    #[error("failed to read from stream")]
    Read(#[from] std::io::Error),

    #[error("decompression error")]
    Decompression(#[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("packet length encoding error")]
    PacketLengthEncode(#[from] protocol::FieldWriteError),

    #[error("encoding error")]
    Encode(#[from] protocol::WriteError),

    #[error("failed to write to stream")]
    Write(#[from] std::io::Error),

    #[error("compressioon error")]
    Compression(#[source] std::io::Error),
}

pub struct Connection {
    stream: BufWriter<TcpStream>,
    packet_buf: Vec<u8>,
    compression_buf: Vec<u8>,
    encryption_buf: Vec<u8>,
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
            encryption_buf: Vec::new(),
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
            self.write_compressed(threshold).await?;
        } else {
            self.write_uncompressed().await?;
        }

        if let Some(cipher) = &mut self.cipher {
            cipher.1.encrypt(&mut self.encryption_buf);
        }

        self.stream.write(&self.encryption_buf).await?;
        self.stream.flush().await?;
        self.encryption_buf.clear();

        trace!("sent packet: {:?}", packet);

        self.packet_buf.clear();

        Ok(())
    }

    async fn write_compressed(&mut self, threshold: usize) -> Result<(), SendError> {
        let length = self.packet_buf.len();

        if length < threshold {
            VarInt(0).write_to(&mut self.compression_buf)?;
            self.compression_buf.extend_from_slice(&self.packet_buf);
        } else {
            trace!("compressing packet of {} bytes", length);
            VarInt(length as i32).write_to(&mut self.compression_buf)?;
            let mut encoder = ZlibEncoder::new(&mut self.compression_buf, Compression::default());
            encoder
                .write_all(&self.packet_buf)
                .map_err(SendError::Compression)?;
            encoder.finish().map_err(SendError::Compression)?;
        }

        let mut buf = [0u8; 5];
        let mut length_bytes = Cursor::new(&mut buf[..]);
        VarInt(self.compression_buf.len() as i32).write_to(&mut length_bytes)?;
        let position = length_bytes.position() as usize;

        self.encryption_buf.extend_from_slice(&buf[..position]);
        self.encryption_buf.extend_from_slice(&self.compression_buf);

        self.compression_buf.clear();

        Ok(())
    }

    async fn write_uncompressed(&mut self) -> Result<(), SendError> {
        let mut buf = [0u8; 5];
        let mut length_bytes = Cursor::new(&mut buf[..]);
        VarInt(self.packet_buf.len() as i32).write_to(&mut length_bytes)?;
        let position = length_bytes.position() as usize;

        self.encryption_buf.extend_from_slice(&buf[..position]);
        self.encryption_buf.extend_from_slice(&self.packet_buf);

        Ok(())
    }
}
