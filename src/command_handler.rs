use std::collections::hash_map::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{Sender};

use crate::app::create_commnand_handler;
use crate::command::Command;
use crate::connect_manager::ConnectionManager;
use crate::message::MessagePtr;
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

    pub fn handle(&mut self, y: MessagePtr) {
        if let Some(command) = self.command.get_mut(&y.mstype) {
            command.exec(y);
        } else {
            let mut net = self.net.clone();

            let send = move |addr: SocketAddr, message: MessagePtr| net.send_message(addr, message);

            if let Some(mut cmd) = create_commnand_handler(y.mstype, send) {
                cmd.exec(y);
                self.command.insert(cmd.get_msgtype(), cmd);
            } else {
                println!("No such Command.");
            }
        }
    }
}
