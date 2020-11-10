use crate::command_handler::CommandType;
use crate::connect_manager::ConnectionManager;
use crate::data::*;
use crate::message::MessagePtr;
use crate::show_message::ShowMessage;

#[warn(unused_assignments)]
pub fn create_commnand_handler(
    _net: &ConnectionManager,
    message: MessagePtr,
) -> Option<CommandType> {
    match message.mstype {
        SHOWMESSAGE => Some(Box::new(ShowMessage::new(message))),
        _ => None,
    }
}
