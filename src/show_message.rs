use crate::command::Command;
use crate::data::SHOWMESSAGE;
use crate::message::{MessagePtr, MSGTYPE};

pub struct ShowMessage {
    message: MessagePtr,
}

impl ShowMessage {
    pub fn new(message: MessagePtr) -> ShowMessage {
        ShowMessage { message }
    }
}

impl Command for ShowMessage {
    fn exec(&mut self) {
        println!("{:?}", self.message);
    }

    fn get_msgtype(&self) -> MSGTYPE {
        SHOWMESSAGE
    }
}
