use super::VarInt;

macro_rules! packet {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $id:literal = $packet:ident {
                    $($field:ident: $typ:ident),*
                    $(,)?
                }
            ),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $packet {
                    $($field: $typ),*
                }
            ),*
        }

        impl crate::protocol::Readable for $name {
            fn read_from(buffer: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::protocol::ProtocolError> {
                match crate::protocol::VarInt::read_from(buffer)?.0 {
                    $(
                        $id => Ok(Self::$packet {
                            $(
                                $field: $typ::read_from(buffer)?,
                            )*
                        }),
                    )*
                    id => Err(crate::protocol::ProtocolError::InvalidPacketId(id)),
                }
            }
        }
    };
}

packet_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum State: VarInt {
        Handshake = 0,
        Status = 1,
        Login = 2,
        Play = 3,
    }
}

pub mod client;
// pub mod server;
