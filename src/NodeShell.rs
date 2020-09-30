use crate::node::Node;
use crate::message::Message;

use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::thread;

pub struct NodeShell {
    pub node: Arc<Node>,
    recv_handle: Option<thread::JoinHandle<()>>,
    check_handle: Option<thread::JoinHandle<()>>,
    pub peer: Option<String>,
}

impl NodeShell {
    pub fn new<A: ToSocketAddrs>(size: usize, addr: A) -> NodeShell {
        let node = Arc::new(Node::new(size, addr));

        let for_receive = node.clone();
        let recv_handle = std::thread::spawn(move || {
            for_receive.start_receive();
        });

        let for_check = node.clone();
        let check_handle = std::thread::spawn(move || {
            for_check.start_check();
        });

        NodeShell {
            node,
            recv_handle: Some(recv_handle),
            check_handle: Some(check_handle),
            peer: None,
        }
    }

    pub fn switchto(&mut self, peer :&str) {
        self.peer = Some(String::from(peer));
    }

    pub fn sendmessage(&self, messsage: &str) {
        if let Some(peer) = self.peer.as_ref() {
            let message = Message::create_man_message(messsage);
            self.node.send_message(&message, peer.as_str());
        } 
    }

    pub fn sendfile(&self, file: &str) {}
}

impl Drop for NodeShell {
    fn drop(&mut self){
        if let Some(handle) = self.recv_handle.take() {
            handle.join().unwrap();
        }
        if let Some(handle) = self.check_handle.take() {
            handle.join().unwrap();
        }
    }
}
