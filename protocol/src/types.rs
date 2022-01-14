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

#[derive(Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Position {
        Position { x, y, z }
    }
}

impl Readable for Position {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Position, ProtocolError> {
        let value = u64::read_from(buffer)?;
        Ok(Position {
            x: (value >> 38) as i32,
            y: ((value << 52) >> 52) as i32,
            z: ((value << 26) >> 38) as i32,
        })
    }
}

impl Writable for Position {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        ((((self.x as u64) & 0x3ffffff) << 38)
            | ((self.y as u64) & 0xfff)
            | (((self.z as u64) & 0x3ffffff) << 12))
            .write_to(buffer)
    }
}
