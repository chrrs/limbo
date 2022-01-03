use std::{
    borrow::Cow,
    io::{Cursor, Write},
};

use crate::{ProtocolError, Readable, Writable};

packet_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum GameMode: i8 {
        Survival = 0,
        Creative = 1,
        Adventure = 2,
        Spectator = 3,
    }
}

impl Readable for Option<GameMode> {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Option<GameMode>, ProtocolError> {
        match i8::read_from(buffer)? {
            -1 => Ok(None),
            // TODO: Code duplication
            0 => Ok(Some(GameMode::Survival)),
            1 => Ok(Some(GameMode::Creative)),
            2 => Ok(Some(GameMode::Adventure)),
            3 => Ok(Some(GameMode::Spectator)),
            id => Err(ProtocolError::InvalidEnumVariant {
                name: Cow::Borrowed("Option<GameMode>"),
                id: Cow::Owned(format!("{}", id)),
            }),
        }
    }
}

impl Writable for Option<GameMode> {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        match self {
            Some(mode) => mode.write_to(buffer),
            None => (-1i8).write_to(buffer),
        }
    }
}
