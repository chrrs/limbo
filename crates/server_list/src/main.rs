use std::{
    io::{Cursor, Read},
    net::TcpListener,
};

use protocol::{fields::varint::VarIntEncoder, Decoder};

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

        let mut buffer = vec![0; length];
        stream
            .read_exact(&mut buffer)
            .expect("failed to read packet");

        let mut cursor = Cursor::new(&buffer[cursor.position() as usize..]);
        let packet_id = VarIntEncoder::decode(&mut cursor).expect("failed to read packet id");
        let protocol_version =
            VarIntEncoder::decode(&mut cursor).expect("failed to read protocol version");

        println!("[ID: {packet_id}, length: {length}] Protocol {protocol_version}")
    }
}
