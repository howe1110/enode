use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;


use crate::framework::{Notify, System, World};
use crate::net::{Node, NodeEvent};

pub struct NetSystem {
    pub sender: SyncSender<NodeEvent>, //node_shell->worker.
    receiver: Option<Receiver<NodeEvent>>,
    inner_sender: SyncSender<Notify>, //connection -> worker.
    pub inner_receiver: Arc<Mutex<Receiver<Notify>>>,
    addr: SocketAddr,
    handle: Option<thread::JoinHandle<()>>,
}

impl NetSystem {
    pub fn new(addr: SocketAddr) -> Self {
        let (sender, receiver) = mpsc::sync_channel(4096);

        let (inner_sender, inner_receiver) = mpsc::sync_channel(4096);

        let inner_receiver = Arc::new(Mutex::new(inner_receiver));

        NetSystem {
            sender,
            receiver: Some(receiver),
            inner_sender,
            inner_receiver,
            addr,
            handle:None,
        }
    }
}

impl System for NetSystem {
    fn setup(&mut self, world: &mut World) {
        world.insert(self.sender.clone());
        world.insert(self.inner_receiver.clone());
    }

    fn run(&mut self) {
        println!("Network starting...");
        if let Some(receiver) = self.receiver.take() {
            let mut node = Node::new(self.addr, receiver, self.inner_sender.clone());
            let handle = std::thread::spawn(move || {
                node.start_polling();
            });
            self.handle = Some(handle);
        }
    }
    fn dispose(&mut self) {
    }
}
