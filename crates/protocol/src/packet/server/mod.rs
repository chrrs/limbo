pub mod status;

#[macro_export]
macro_rules! encodable_packet {
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

        impl $crate::Encodable for $name {
            fn encode(&self, w: &mut impl std::io::Write) -> Result<(), $crate::EncodingError> {
                #[allow(unused_imports)]
                use $crate::{Encoder, Encodable};

                match self {
                    $( Self::$packet(value) => {
                        $crate::fields::varint::VarIntEncoder::encode($id, w)?;
                        value.encode(w)
                    }, )*
                }
            }
        }
    };
}
