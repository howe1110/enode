use serde::{Deserialize, Serialize};

use crate::net::NODEIDTYPE;

pub type MSGTYPE = u16;

pub type MessagePtr = Box<Message>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub sendid: NODEIDTYPE,
    pub recvid: NODEIDTYPE,
    pub taskid: u16, //表示一个任务，可以有多个消息组成，通过hash方式将相同任务指向同一个线程
    pub mstype: u16,
    pub subtype: u16,
    pub msglen: u16,
    pub msgdata: Vec<u8>,
}
