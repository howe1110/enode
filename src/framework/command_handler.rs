use std::collections::hash_map::HashMap;
use std::sync::mpsc::{SyncSender};

use crate::framework::command::Command;
use crate::net::connect_manager::ConnectionManager;
use crate::net::EMessagePtr;
use crate::net::NodeEvent;
use crate::app::create_commnand_handler;

pub type CommandType = Box<dyn Command + Send>;

pub struct CommandHandler {
    net: ConnectionManager,
    command: HashMap<u16, CommandType>,
}

impl CommandHandler {
    pub fn new(sender: SyncSender<NodeEvent>) -> Self {
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
