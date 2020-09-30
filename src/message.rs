use serde::{Deserialize, Serialize};

use crate::data::UserMessage;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub mstype: u16,
    pub msglen: u16,
    pub msgdata: Vec<u8>,
}

impl Message {
    pub fn create_man_message(message:&str) -> Message {
        Message {
            mstype:UserMessage,
            msglen:message.len() as u16,
            msgdata:message.as_bytes().to_vec(),
        }
    }
}