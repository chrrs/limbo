pub mod handshake;
pub mod status;

#[macro_export]
macro_rules! decodable_packet {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $id:literal = $packet:ident($strct:ident) ),*
            $(,)?
        }
    } => {
        $(#[$meta])*
        $vis enum $name {
            $( $packet($strct) ),*
        }

        impl $crate::Decodable for $name {
            fn decode(r: &mut impl std::io::Read) -> Result<Self, $crate::DecodingError> {
                #[allow(unused_imports)]
                use $crate::{Decoder, Decodable};

                match $crate::fields::varint::VarIntEncoder::decode(r)? {
                    $( $id => $strct::decode(r).map(Self::$packet), )*
                    id => Err($crate::DecodingError::InvalidPacketId(id)),
                }
            }
        }
    };
}
