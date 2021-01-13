use crate::app::SHOWMESSAGE;
use crate::framework::Command;
use crate::net::{EMessagePtr, MSGTYPE};

pub struct ShowMessage {}

impl ShowMessage {
    pub fn new() -> ShowMessage {
        ShowMessage {}
    }
}

impl Command for ShowMessage {
    fn exec(&mut self, _message: EMessagePtr) {}

    fn get_msgtype(&self) -> MSGTYPE {
        SHOWMESSAGE
    }
}
