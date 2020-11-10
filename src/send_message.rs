use std::net::SocketAddr;

use crate::command::Command;
use crate::connect_manager::ConnectionManager;
use crate::data::SENDMESSAGE;
use crate::message::{MessagePtr, MSGTYPE};

pub struct SendMessage<'a> {
    net: &'a mut ConnectionManager,
    addr: SocketAddr,
    message: Option<MessagePtr>,
}

impl<'a> SendMessage<'a> {
    pub fn new(net: &mut ConnectionManager, addr: SocketAddr, message: MessagePtr) -> SendMessage {
        SendMessage {
            net,
            addr,
            message: Some(message),
        }
    }
}

impl<'a> Command for SendMessage<'a> {
    fn exec(&mut self) {
        if let Some(message) = self.message.take() {
            self.net.send_message(self.addr, message);
        }
    }

    fn get_msgtype(&self) -> MSGTYPE {
        SENDMESSAGE
    }
}
