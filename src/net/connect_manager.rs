use std::sync::mpsc::SyncSender;

use crate::net::emessage::EMessagePtr;
use crate::net::node::NodeEvent;

#[derive(Clone, Debug)]
pub struct ConnectionManager {
    sender: SyncSender<NodeEvent>, //node_shell->worker.
}

impl ConnectionManager {
    pub fn new(sender: SyncSender<NodeEvent>) -> ConnectionManager {
        ConnectionManager { sender }
    }

    pub fn send_message(&mut self, message: EMessagePtr) {
        let message = NodeEvent::Message(message);
        self.sender.send(message).unwrap();
    }
}
