use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::{self, sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;

use crate::emessage::EMessage;
use crate::message::Message;
use crate::node::{ConnectEvent, Node};

pub struct NodeShell {
    pub peer: Option<String>,
    sender: SyncSender<ConnectEvent>,
    connections: HashMap<SocketAddr, SyncSender<Message>>,
    handle: thread::JoinHandle<()>,
}

impl NodeShell {
    pub fn new(size: usize, addr: String) -> NodeShell {
        let (sender, receiver) = sync_channel(0);
        let handle = std::thread::spawn(move || {
            let mut node = Node::new(size, addr, receiver);
            node.start_polling();
        });

        let connections = HashMap::new();

        NodeShell {
            peer: None,
            sender,
            connections,
            handle,
        }
    }

    pub fn switchto(&mut self, peer: &str) {
        self.peer = Some(String::from(peer));
    }

    pub fn sendmessage(&mut self, user_message: &str, addr: SocketAddr) {
        if !self.connections.contains_key(&addr) {
            let (message_sender, message_receiver) = sync_channel(0);
            let connect = ConnectEvent::new(addr, message_receiver);
            self.sender.send(connect);
            self.connections.insert(addr, message_sender);
        }
        let connection = self.connections.get(&addr);
        if let Some(sender) = connection.as_ref() {
            let message = Message::create_man_message(user_message);
            sender.send(message);
        }
    }

    pub fn sendfile(&self, file: &str) {}
}

impl Drop for NodeShell {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    const SERVER_ADDR1: &'static str = "127.0.0.1:30022";
    const SERVER_ADDR2: &'static str = "127.0.0.1:30023";

    const TESTSTRING:&'static str = "102321312321321321434325098dsfljfldsjflk34jridsfjdlsjfldsfj034234980-32423jldsjflsdjfldsjfldsjfldsjfldsjfldsjfldjfdlfjdlkfjdlfjdlskfjdlfjdlsfjdlfjdsfjdsfjdlfjdlfjdlfjdljfdlkfjlkdjfkldjflkdjfkdjfdsjf";
    #[test]
    fn send_1_k_message() {
        let mut node_1 = NodeShell::new(2, SERVER_ADDR1.parse().unwrap());

        for _ in 0..10240 {
            node_1.sendmessage(TESTSTRING, SERVER_ADDR2.parse().unwrap());
        }

        thread::sleep_ms(50000);
    }
}
