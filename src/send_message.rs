use std::net::SocketAddr;

use crate::command::Command;
use crate::data::{SENDMESSAGE, SHOWMESSAGE};
use crate::message::{MessagePtr, MSGTYPE};

pub struct SendMessage<F>
where
    F: FnMut(SocketAddr, MessagePtr) + Send + 'static,
{
    send: F,
}

impl<F> SendMessage<F>
where
    F: FnMut(SocketAddr, MessagePtr) + Send + 'static,
{
    pub fn new(send: F) -> SendMessage<F> {
        SendMessage { send }
    }
}

impl<F> Command for SendMessage<F>
where
    F: FnMut(SocketAddr, MessagePtr) + Send + 'static,
{
    fn exec(&mut self, message: MessagePtr) {
        let mut message = message;
        message.mstype = SHOWMESSAGE;
        (self.send)(message.addr, message);
    }

    fn get_msgtype(&self) -> MSGTYPE {
        SENDMESSAGE
    }
}
