use std::{io::Read, net::TcpListener};

use protocol::{
    fields::varint::VarIntEncoder, packet::client::handshake::Handshake, Decodable, DecodeBuffer,
    Decoder,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").expect("failed to open TCP listener");

    for stream in listener.incoming() {
        let mut stream = stream.expect("failed to open stream");

        let mut buffer = [0; 5];
        stream
            .peek(&mut buffer)
            .expect("failed to peek for packet length");
        let mut decode_buffer = DecodeBuffer::new(&buffer[..]);
        let length = VarIntEncoder::decode(&mut decode_buffer)
            .expect("failed to decode packet length") as usize;

        let mut buffer = vec![0; length + decode_buffer.position()];
        stream
            .read_exact(&mut buffer)
            .expect("failed to read packet");

        let mut decode_buffer = DecodeBuffer::new(&buffer[decode_buffer.position()..]);
        VarIntEncoder::decode(&mut decode_buffer).expect("failed to decode packet id");
        let handshake = Handshake::decode(&mut decode_buffer);

        println!("{handshake:?}")
    }
}
