use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::message::MessagePtr;
use crate::node::MessageEvent;
use crate::node::NodeEvent;

#[derive(Clone, Debug)]
pub struct ConnectionManager {
    sender: Sender<NodeEvent>, //node_shell->worker.
}

impl ConnectionManager {
    pub fn new(sender: Sender<NodeEvent>) -> ConnectionManager {
        ConnectionManager { sender }
    }

    pub fn send_message(&mut self, addr: SocketAddr, message: MessagePtr) {
        let message = NodeEvent::Message(MessageEvent::new(0, addr, message));
        self.sender.send(message).unwrap();
    }
}
