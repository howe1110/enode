use crate::data::SHOWMESSAGE;
use serde::{Deserialize, Serialize};

pub type MSGTYPE = u16;

pub type MessagePtr = Box<Message>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub sendid: u16,
    pub recvid: u16,
    pub taskid: u16, //表示一个任务，可以有多个消息组成，通过hash方式将相同任务指向同一个线程
    pub mstype: u16,
    pub subtype: u16,
    pub msglen: u16,
    pub msgdata: Vec<u8>,
}

impl Message {
    pub fn create_man_message(message: &str) -> Message {
        Message {
            sendid: 0,
            recvid: 0,
            taskid: 0,
            mstype: SHOWMESSAGE,
            subtype: SHOWMESSAGE,
            msglen: message.len() as u16,
            msgdata: message.as_bytes().to_vec(),
        }
    }
}
