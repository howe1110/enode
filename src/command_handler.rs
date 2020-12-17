use std::collections::hash_map::HashMap;
use std::sync::mpsc::{Sender};

use crate::app::create_commnand_handler;
use crate::command::Command;
use crate::connect_manager::ConnectionManager;
use crate::emessage::EMessagePtr;
use crate::node::NodeEvent;

pub type CommandType = Box<dyn Command + Send>;

pub struct CommandHandler {
    net: ConnectionManager,
    command: HashMap<u16, CommandType>,
}

impl CommandHandler {
    pub fn new(sender: Sender<NodeEvent>) -> Self {
        let command = HashMap::new();

        let net = ConnectionManager::new(sender);

        CommandHandler { command, net }
    }

    pub fn handle(&mut self, y: EMessagePtr) {
        if let Some(command) = self.command.get_mut(&y.payload.mstype) {
            command.exec(y);
        } else {
            let mut net = self.net.clone();

            let send = move |message: EMessagePtr| net.send_message(message);

            if let Some(mut cmd) = create_commnand_handler(y.payload.mstype, send) {
                cmd.exec(y);
                self.command.insert(cmd.get_msgtype(), cmd);
            } else {
                println!("No such Command.");
            }
        }
    }
}
