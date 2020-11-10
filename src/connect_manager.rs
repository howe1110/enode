use std::collections::hash_map::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{sync_channel, SyncSender};

use crate::message::MessagePtr;
use crate::node::ConnectEvent;
use crate::node::NodeEvent;

pub struct ConnectionManager {
    pub worker_id: usize,
    sender: SyncSender<NodeEvent>, //node_shell->worker.
    connections: HashMap<SocketAddr, SyncSender<MessagePtr>>,
}

impl ConnectionManager {
    pub fn new(worker_id: usize, sender: SyncSender<NodeEvent>) -> ConnectionManager {
        let connections = HashMap::new();
        ConnectionManager {
            worker_id,
            sender,
            connections,
        }
    }
    pub fn connect(&mut self, addr: SocketAddr) -> &SyncSender<MessagePtr> {
        if self.connections.contains_key(&addr) {
            let (message_sender, message_receiver) = sync_channel(0);
            self.connections.insert(addr, message_sender);

            let connect =
                NodeEvent::Connect(ConnectEvent::new(self.worker_id, addr, message_receiver));

            self.sender.send(connect).unwrap();
        }

        if let Some(conn) = self.connections.get(&addr) {
            conn
        } else {
            panic!("connect to {} error.", addr); //Not occured.
        }
    }

    pub fn send_message(&mut self, addr: SocketAddr, message: MessagePtr) {
        let conn = self.connect(addr);
        conn.send(message).unwrap();
    }
}
