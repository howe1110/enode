use crate::command::Command;
use crate::data::{SENDMESSAGE, SHOWMESSAGE};
use crate::emessage::EMessagePtr;
use crate::message::MSGTYPE;

pub struct SendMessage<F>
where
    F: FnMut(EMessagePtr) + Send + 'static,
{
    send: F,
}

impl<F> SendMessage<F>
where
    F: FnMut(EMessagePtr) + Send + 'static,
{
    pub fn new(send: F) -> SendMessage<F> {
        SendMessage { send }
    }
}

impl<F> Command for SendMessage<F>
where
    F: FnMut(EMessagePtr) + Send + 'static,
{
    fn exec(&mut self, message: EMessagePtr) {
        let mut message = message;
        message.payload.mstype = SHOWMESSAGE;
        (self.send)(message);
    }

    fn get_msgtype(&self) -> MSGTYPE {
        SENDMESSAGE
    }
}


