use std::{
    collections::HashMap,
    io::{Read, Write},
};

use uuid::Uuid;

use crate::{
    chat::Message,
    io::BooleanPrefixedOption,
    types::{Pose, Position},
    FieldReadError, FieldWriteError, PacketField, VarInt,
};

#[derive(Debug)]
pub enum MetaType {
    Byte(u8),
    VarInt(VarInt),
    Float(f32),
    String(String),
    Chat(Message),
    OptionalChat(Option<Message>),
    // Slot,
    Boolean(bool),
    // Rotation,
    Position(Position),
    OptionalPosition(Option<Position>),
    // Direction,
    OptionalUuid(Option<Uuid>),
    // OptionalBlockId,
    // NBT,
    // Particle,
    // VillagerData,
    OptionalVarInt(Option<VarInt>),
    Pose(Pose),
}

impl PacketField for MetaType {
    fn read_from(buffer: &mut dyn Read) -> Result<MetaType, FieldReadError> {
        match VarInt::read_from(buffer)
            .map_err(|e| FieldReadError::SubField("type", Box::new(e)))?
            .0
        {
            0 => Ok(MetaType::Byte(u8::read_from(buffer)?)),
            1 => Ok(MetaType::VarInt(VarInt::read_from(buffer)?)),
            2 => Ok(MetaType::Float(f32::read_from(buffer)?)),
            3 => Ok(MetaType::String(String::read_from(buffer)?)),
            4 => Ok(MetaType::Chat(Message::read_from(buffer)?)),
            5 => Ok(MetaType::OptionalChat(
                BooleanPrefixedOption::read_from(buffer)?.0,
            )),
            7 => Ok(MetaType::Boolean(bool::read_from(buffer)?)),
            9 => Ok(MetaType::Position(Position::read_from(buffer)?)),
            10 => Ok(MetaType::OptionalPosition(
                BooleanPrefixedOption::read_from(buffer)?.0,
            )),
            12 => Ok(MetaType::OptionalUuid(
                BooleanPrefixedOption::read_from(buffer)?.0,
            )),
            17 => {
                let value = VarInt::read_from(buffer)?.0;
                Ok(MetaType::OptionalVarInt(if value == 0 {
                    None
                } else {
                    Some(VarInt(value - 1))
                }))
            }
            18 => Ok(MetaType::Pose(Pose::read_from(buffer)?)),
            id => Err(FieldReadError::InvalidEnumId(format!("{}", id))),
        }
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        match self {
            MetaType::Byte(value) => {
                0u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::VarInt(value) => {
                1u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::Float(value) => {
                2u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::String(value) => {
                3u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::Chat(value) => {
                4u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::OptionalChat(value) => {
                5u8.write_to(buffer)?;
                BooleanPrefixedOption(value.as_ref()).write_to(buffer)
            }
            MetaType::Boolean(value) => {
                7u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::Position(value) => {
                9u8.write_to(buffer)?;
                value.write_to(buffer)
            }
            MetaType::OptionalPosition(value) => {
                10u8.write_to(buffer)?;
                BooleanPrefixedOption(value.as_ref()).write_to(buffer)
            }
            MetaType::OptionalUuid(value) => {
                12u8.write_to(buffer)?;
                BooleanPrefixedOption(value.as_ref()).write_to(buffer)
            }
            MetaType::OptionalVarInt(value) => {
                17u8.write_to(buffer)?;
                match value {
                    Some(value) => VarInt(value.0 + 1).write_to(buffer),
                    None => VarInt(0).write_to(buffer),
                }
            }
            MetaType::Pose(value) => {
                9u8.write_to(buffer)?;
                value.write_to(buffer)
            }
        }
    }
}

macro_rules! indices {
    ($(
        $value:expr => $name:ident
    ),*$(,)?) => {
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub enum MetaIndex {
            $($name),*
        }

        impl MetaIndex {
            pub const fn id(&self) -> u8 {
                match self {
                    $(
                        MetaIndex::$name => $value,
                    )*
                }
            }
        }
    };
}

indices!(
    // Entity
    0 => Flags,
    1 => AirTicks,
    2 => CustomName,
    3 => CustomNameVisible,
    4 => Silent,
    5 => NoGravity,
    6 => Pose,
    7 => TicksFrozen,

    // Living Entity
    8 => HandState,
    9 => Health,
    10 => PotionEffectColor,
    11 => PotionEffectAmbient,
    12 => Arrows,
    13 => BeeStingers,
    14 => BedLocation,

    // Player
    15 => AdditionalHearts,
    16 => Score,
    17 => SkinParts,
    18 => MainHand,
    19 => LeftShoulderEntity,
    20 => RightShoulderEntity,
);

#[derive(Debug)]
pub struct EntityMetadata(HashMap<u8, MetaType>);

impl EntityMetadata {
    pub fn new() -> EntityMetadata {
        EntityMetadata(HashMap::new())
    }

    pub fn with(mut self, index: MetaIndex, value: MetaType) -> EntityMetadata {
        self.add(index, value);
        self
    }

    pub fn add(&mut self, index: MetaIndex, value: MetaType) {
        self.0.insert(index.id(), value);
    }
}

impl PacketField for EntityMetadata {
    fn read_from(buffer: &mut dyn Read) -> Result<EntityMetadata, FieldReadError> {
        let mut map = HashMap::new();

        loop {
            let i = u8::read_from(buffer)?;

            if i == 0xff {
                break;
            }

            map.insert(i, MetaType::read_from(buffer)?);
        }

        Ok(EntityMetadata(map))
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        for (i, value) in &self.0 {
            i.write_to(buffer)?;
            value.write_to(buffer)?;
        }

        0xffu8.write_to(buffer)
    }
}
