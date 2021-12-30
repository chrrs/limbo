use std::{
    borrow::Cow,
    io::{Cursor, Write},
};

use serde::{Deserialize, Serialize};

use crate::{ProtocolError, Readable, Writable};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    text: Cow<'static, str>,
}

impl Message {
    pub fn new<S: Into<Cow<'static, str>>>(text: S) -> Message {
        Message { text: text.into() }
    }
}

impl Readable for Message {
    fn read_from(buffer: &mut Cursor<&[u8]>) -> Result<Message, ProtocolError> {
        Ok(serde_json::from_str(&String::read_from(buffer)?)?)
    }
}

impl Writable for Message {
    fn write_to(&self, buffer: &mut dyn Write) -> Result<(), ProtocolError> {
        serde_json::to_string(self)?.write_to(buffer)
    }
}
