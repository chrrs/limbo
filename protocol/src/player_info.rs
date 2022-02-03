use std::io::{Read, Write};

use uuid::Uuid;

use crate::{
    chat::Message,
    io::{BooleanPrefixedOption, VarIntPrefixedVec},
    types::GameMode,
    FieldReadError, FieldWriteError, PacketField, VarInt,
};

#[derive(Debug)]
pub enum PlayerInfo {
    AddPlayer(VarIntPrefixedVec<AddPlayerAction>),
}

packet_field! {
    #[derive(Debug)]
    pub struct AddPlayerAction {
        pub uuid: Uuid,
        pub name: String,
        pub properties: VarIntPrefixedVec<AddPlayerProperty>,
        pub game_mode: GameMode,
        pub ping: VarInt,
        pub display_name: BooleanPrefixedOption<Message>,
    }
}

packet_field! {
    #[derive(Debug)]
    pub struct AddPlayerProperty {
        pub name: String,
        pub value: String,
        pub signature: BooleanPrefixedOption<String>,
    }
}

impl PacketField for PlayerInfo {
    fn read_from(buffer: &mut dyn Read) -> Result<PlayerInfo, FieldReadError> {
        match VarInt::read_from(buffer)? {
            VarInt(0) => Ok(PlayerInfo::AddPlayer(VarIntPrefixedVec::read_from(buffer)?)),
            id => Err(FieldReadError::InvalidEnumId(format!("{:?}", id))),
        }
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        match self {
            PlayerInfo::AddPlayer(action) => {
                VarInt(0).write_to(buffer)?;
                action.write_to(buffer)?;
            }
        }

        Ok(())
    }
}
