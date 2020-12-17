use crate::command::Command;
use crate::data::SHOWMESSAGE;
use crate::emessage::{EMessagePtr };
use crate::message::MSGTYPE;

pub struct ShowMessage {}

impl ShowMessage {
    pub fn new() -> ShowMessage {
        ShowMessage {}
    }
}

impl Command for ShowMessage {
    fn exec(&mut self, _message: EMessagePtr) {
    }

    fn get_msgtype(&self) -> MSGTYPE {
        SHOWMESSAGE
    }
}
