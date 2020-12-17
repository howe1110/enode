use crate::emessage::{EMessagePtr};

pub trait Command {
    fn get_msgtype(&self) -> u16;
    fn exec(&mut self, message: EMessagePtr);
}
