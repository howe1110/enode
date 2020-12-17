use std::net::SocketAddr;

use crate::message::{Message};


pub type EMessagePtr = Box<EMessage>;

pub struct EMessage {
    pub addr: SocketAddr,
    pub payload: Box<Message>,
}

impl EMessage {
    pub fn construct_emessage(addr: SocketAddr, message: Message) -> EMessage {
        EMessage {
            addr,
            payload: Box::new(message),
        }
    }
}
