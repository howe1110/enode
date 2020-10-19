use crate::message::Message;
use crate::node::Node;

use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::sync::atomic::AtomicBool;

pub struct NodeShell {
    pub node: Arc<Node>,
    recv_handle: Option<thread::JoinHandle<()>>,
    check_handle: Option<thread::JoinHandle<()>>,
    pub peer: Option<String>,
}

impl NodeShell {
    pub fn new<A: ToSocketAddrs>(size: usize, addr: A) -> NodeShell {
        let node = Arc::new(Node::new(size, addr));

        let mut running = AtomicBool::new(true);


        let for_receive = node.clone();
        let recv_handle = std::thread::spawn(move || loop {
            for_receive.start_receive();
        });

        let for_check = node.clone();
        let check_handle = std::thread::spawn(move || loop {
            thread::sleep(std::time::Duration::from_millis(10));
            for_check.start_check();
        });

        NodeShell {
            node,
            recv_handle: Some(recv_handle),
            check_handle: Some(check_handle),
            peer: None,
        }
    }

    pub fn switchto(&mut self, peer: &str) {
        self.peer = Some(String::from(peer));
    }

    pub fn sendmessage(&self, messsage: &str) {
        if let Some(peer) = self.peer.as_ref() {
            let message = Message::create_man_message(messsage);
            self.node.send_usermessage(&message, peer.as_str());
        }
    }

    pub fn sendfile(&self, file: &str) {}
}

impl Drop for NodeShell {
    fn drop(&mut self) {
        if let Some(handle) = self.recv_handle.take() {
            //handle.join().unwrap();
        }
        if let Some(handle) = self.check_handle.take() {
            //handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTSTRING:&str = "102321312321321321434325098dsfljfldsjflk34jridsfjdlsjfldsfj034234980-32423jldsjflsdjfldsjfldsjfldsjfldsjfldsjfldjfdlfjdlkfjdlfjdlskfjdlfjdlsfjdlfjdsfjdsfjdlfjdlfjdlfjdljfdlkfjlkdjfkldjflkdjfkdjfdsjf";
    #[test]
    fn send_1_k_message() {
        {
            let locaddr = String::from("127.0.0.1:35100");
            let peeraddr = String::from("127.0.0.1:35200");
            let mut loc = NodeShell::new(2, locaddr.clone());
            let mut peer = NodeShell::new(1, peeraddr.clone());
            loc.peer = Some(peeraddr.clone());
            peer.peer = Some(locaddr.clone());

            println!("Test string len {}", TESTSTRING.len());
            for _ in 0.. {
                loc.sendmessage(TESTSTRING);
            }
            thread::sleep(std::time::Duration::from_millis(10000));
            assert_eq!(peer.node.receive_count(locaddr.as_str()), loc.node.send_count(peeraddr.as_str()));
        }
    }
}