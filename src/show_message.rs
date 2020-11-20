use crate::command::Command;
use crate::data::SHOWMESSAGE;
use crate::message::{MessagePtr, MSGTYPE};

pub struct ShowMessage {}

impl ShowMessage {
    pub fn new() -> ShowMessage {
        ShowMessage {}
    }
}

impl Command for ShowMessage {
    fn exec(&mut self, _message: MessagePtr) {
    }

    fn get_msgtype(&self) -> MSGTYPE {
        SHOWMESSAGE
    }
}
