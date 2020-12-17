use crate::addr_server::AddrServer;
use crate::command_handler::CommandType;
use crate::data::NODEIDTYPE;
use crate::data::*;
use crate::emessage::EMessagePtr;
use crate::send_message::SendMessage;
use crate::show_message::ShowMessage;

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
        ADDRMESSAGE => Some(Box::new(AddrServer::new(send))),
        _ => None,
    }
}
