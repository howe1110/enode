use std::sync::mpsc::Sender;

use crate::emessage::EMessagePtr;
use crate::node::NodeEvent;

#[derive(Clone, Debug)]
pub struct ConnectionManager {
    sender: Sender<NodeEvent>, //node_shell->worker.
}

impl ConnectionManager {
    pub fn new(sender: Sender<NodeEvent>) -> ConnectionManager {
        ConnectionManager { sender }
    }

    pub fn send_message(&mut self, message: EMessagePtr) {
        let message = NodeEvent::Message(message);
        self.sender.send(message).unwrap();
    }
}
