use std::{fmt, io::Cursor};

use bytes::{Buf, BytesMut};
use protocol::{fields::varint::VarIntEncoder, Decodable, Decoder, DecodingError};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufWriter},
    net::TcpStream,
};
use tracing::trace;

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error("couldn't read packet length")]
    PacketLengthError(#[source] DecodingError),

    #[error("failed to decode packet")]
    Decode(#[source] DecodingError),

    #[error("failed to read from stream")]
    Read(#[source] std::io::Error),

    #[error("connection closed")]
    ConnectionClosed,

    #[error("connection reset by peer")]
    ConnectionResetByPeer,
}

pub struct Connection {
    stream: BufWriter<TcpStream>,
    receive_buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            receive_buffer: BytesMut::new(),
        }
    }

    pub async fn receive_packet<P: fmt::Debug + Decodable>(&mut self) -> Result<P, ReceiveError> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                return Ok(packet);
            }

            let bytes_read = self
                .stream
                .read_buf(&mut self.receive_buffer)
                .await
                .map_err(ReceiveError::Read)?;

            if bytes_read == 0 {
                if self.receive_buffer.is_empty() {
                    return Err(ReceiveError::ConnectionClosed);
                } else {
                    return Err(ReceiveError::ConnectionResetByPeer);
                }
            }
        }
    }

    pub fn parse_packet<P: fmt::Debug + Decodable>(&mut self) -> Result<Option<P>, ReceiveError> {
        let mut cursor = Cursor::new(&self.receive_buffer);
        let Ok(length) = VarIntEncoder::decode(&mut cursor) else {
            return Ok(None);
        };

        let prefix_size = cursor.position() as usize;
        if self.receive_buffer.len() < length as usize + prefix_size {
            return Ok(None);
        }

        let packet = P::decode(&mut cursor);

        self.receive_buffer.advance(length as usize + prefix_size);

        match packet {
            Ok(packet) => {
                trace!("received packet: {packet:?}");
                Ok(Some(packet))
            }
            Err(e) => Err(ReceiveError::Decode(e)),
        }
    }
}
