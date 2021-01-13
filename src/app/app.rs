use crate::framework::CommandType;
use crate::app::*;
use crate::net::EMessagePtr;
use crate::app::SendMessage;
use crate::app::ShowMessage;
use crate::net::NODEIDTYPE;

pub static mut NODEID: NODEIDTYPE = 0;

pub fn set_message_no(id: NODEIDTYPE) {
    unsafe { NODEID = id };
}

pub fn get_node_id() -> NODEIDTYPE {
    unsafe { NODEID }
}

#[warn(unused_assignments)]
pub fn create_commnand_handler<T>(msgtype: u16, send: T) -> Option<CommandType>
where
    T: FnMut(EMessagePtr) + Send + 'static,
{
    match msgtype {
        SHOWMESSAGE => Some(Box::new(ShowMessage::new())),
        SENDMESSAGE => Some(Box::new(SendMessage::new(send))),
        _ => None,
    }
}
