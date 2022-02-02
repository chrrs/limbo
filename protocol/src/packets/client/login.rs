use crate::io::VarIntPrefixedVec;

packet! {
    #[derive(Debug)]
    pub enum ClientLoginPacket {
        0x00 = Start {
            name: String,
        },
        0x01 = EncryptionResponse {
            shared_secret: VarIntPrefixedVec<u8>,
            verify_token: VarIntPrefixedVec<u8>,
        }
    }
}
