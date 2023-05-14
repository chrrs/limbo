use std::{
    io::{Cursor, Read},
    net::TcpListener,
};

use protocol::{
    fields::varint::VarIntEncoder, packet::client::handshake::HandshakePacket, Decodable, Decoder,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").expect("failed to open TCP listener");

    for stream in listener.incoming() {
        let mut stream = stream.expect("failed to open stream");

        let mut buffer = [0; 5];
        stream
            .peek(&mut buffer)
            .expect("failed to peek for packet length");
        let mut cursor = Cursor::new(&buffer[..]);
        let length =
            VarIntEncoder::decode(&mut cursor).expect("failed to decode packet length") as usize;

        let mut buffer = vec![0; length + cursor.position() as usize];
        stream
            .read_exact(&mut buffer)
            .expect("failed to read packet");

        let mut cursor = Cursor::new(&buffer[cursor.position() as usize..]);

        println!("{:?}", HandshakePacket::decode(&mut cursor));
    }
}
