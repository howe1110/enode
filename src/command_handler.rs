use std::collections::hash_map::HashMap;
use std::sync::mpsc::SyncSender;

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
    pub fn new(worker_id: usize, sender: SyncSender<NodeEvent>) -> CommandHandler {
        let net = ConnectionManager::new(worker_id, sender);
        let command = HashMap::new();
        CommandHandler { net, command }
    }

    pub fn handle(&mut self, y: MessagePtr) {
        if let Some(command) = self.command.get_mut(&y.mstype) {
            command.exec();
        } else {
            if let Some(mut cmd) = create_commnand_handler(&self.net, y) {
                cmd.exec();
                self.command.insert(cmd.get_msgtype(), cmd);
            } else {
                println!("No such Command.");
            }
        }
    }
}
