use uuid::Uuid;

use crate::{chat::Message, io::VarIntPrefixedVec, VarInt};

packet! {
    #[derive(Debug)]
    pub enum ServerLoginPacket {
        0x00 = Disconnect {
            reason: Message,
        },
        0x01 = EncryptionRequest {
            server_id: String,
            public_key: VarIntPrefixedVec<u8>,
            verify_token: VarIntPrefixedVec<u8>,
        },
        0x02 = Success {
            uuid: Uuid,
            name: String,
        },
        0x03 = SetCompression {
            threshold: VarInt,
        },
    }
}
