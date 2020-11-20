use std::net::SocketAddr;

use crate::command_handler::CommandType;
use crate::data::*;
use crate::message::MessagePtr;
use crate::send_message::SendMessage;
use crate::show_message::ShowMessage;

#[warn(unused_assignments)]
pub fn create_commnand_handler<T>(msgtype: u16, send: T) -> Option<CommandType>
where
    T: FnMut(SocketAddr, MessagePtr) + Send + 'static,
{
    match msgtype {
        SHOWMESSAGE => Some(Box::new(ShowMessage::new())),
        SENDMESSAGE => Some(Box::new(SendMessage::new(send))),
        _ => None,
    }
}
