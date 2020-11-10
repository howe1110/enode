use std::net::SocketAddr;

use crate::message::Message;

pub type CmdIdType = u32;

pub type

pub struct IMessage {
    cmd: CmdIdType,
    pub addr: Option<SocketAddr>,
    pub payload: Box<Message>,
}

impl IMessage {
    pub fn construct_imessage(addr: SocketAddr, message: Message) -> IMessage {
        IMessage {
            cmd: 
            addr: Some(addr),
            payload: Box::new(message),
        }
    }
}
