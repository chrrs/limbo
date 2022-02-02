use std::{
    borrow::Cow,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::{FieldReadError, FieldWriteError, PacketField};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    text: Cow<'static, str>,
}

impl Message {
    pub fn new<S: Into<Cow<'static, str>>>(text: S) -> Message {
        Message { text: text.into() }
    }
}

impl PacketField for Message {
    fn read_from(buffer: &mut dyn Read) -> Result<Message, FieldReadError> {
        Ok(serde_json::from_str(&String::read_from(buffer)?)?)
    }

    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), FieldWriteError> {
        serde_json::to_string(self)?.write_to(buffer)
    }
}
