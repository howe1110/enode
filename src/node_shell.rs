use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::mpsc::{self, sync_channel, Sender, SyncSender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::message::Message;
use crate::node::{Node, NodeEvent};
use crate::worker::{Notify, Worker};

fn send2peer(paras: &[&str], sender: Sender<Notify>) -> bool {
    if paras.len() != 2 {
        return false;
    }

    if let Ok(peer) = paras[0].parse::<SocketAddr>() {
        println!("{:?}", peer);

        let message = Message::create_man_message(peer, paras[1]);

        sender
            .send(Notify::NewJob {
                y: Box::new(message),
            })
            .unwrap();
    }

    true
}

fn init_user_fn() -> HashMap<String, fn(&[&str], Sender<Notify>) -> bool> {
    let mut userfn = HashMap::new();
    userfn.insert(
        String::from("sw"),
        send2peer as fn(&[&str], Sender<Notify>) -> bool,
    );
    userfn
}

pub struct NodeShell {
    pub peer: Option<String>,
    sender: Sender<NodeEvent>, //node_shell->worker.
    inner_sender: Sender<Notify>,  //connection -> worker.
    handle: Option<thread::JoinHandle<()>>,
    pub workers: Vec<Worker>,
}

impl NodeShell {
    pub fn new(size: usize, addr: SocketAddr) -> NodeShell {
        let (sender, receiver) = mpsc::channel();

        let (inner_sender, inner_receiver) = mpsc::channel();

        let mut node = Node::new(addr, receiver, inner_sender.clone());

        let inner_receiver = Arc::new(Mutex::new(inner_receiver));

        let handle = std::thread::spawn(move || {
            node.start_polling();
        });

        let mut workers = Vec::with_capacity(size);
        for id in 0..workers.capacity() {
            println!("create worker {}", id);
            workers.push(Worker::new(id, sender.clone(), inner_receiver.clone()));
        }

        let handle = Some(handle);

        NodeShell {
            peer: None,
            sender,
            inner_sender,
            handle,
            workers,
        }
    }

    pub fn polling(&mut self) {
        let userfns = init_user_fn();
        loop {
            let mut input = String::new();

            print!("==={}>", self.peer.as_ref().unwrap_or(&String::from("")));

            io::stdout().flush().unwrap();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let input = input.trim();

            if let Ordering::Equal = input.cmp(&"quit".to_string()) {
                break;
            }
            let paras: Vec<&str> = input.split_whitespace().collect();
            if paras.len() == 0 {
                continue;
            }
            if let Some(pfn) = userfns.get(paras[0]) {
                println!("user function is {}", paras[0]);
                pfn(&paras[1..paras.len()], self.inner_sender.clone());
                continue;
            }
        }
    }

    pub fn switchto(&mut self, peer: &str) {
        self.peer = Some(String::from(peer));
    }

    pub fn send_message(&mut self, message: &str, addr: &str) {
        let message = Message::create_send_message(addr.parse().unwrap(), message);
        self.inner_sender
            .send(Notify::NewJob {
                y: Box::new(message),
            })
            .unwrap();
    }

    pub fn sendfile(&self, file: &str) {}
}

impl Drop for NodeShell {
    fn drop(&mut self) {
        let terminate = NodeEvent::Terminate;
        self.sender.send(terminate).unwrap();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }

        //
        println!("Sending terminate message to all workers.");
        for _ in &mut self.workers {
            self.inner_sender.send(Notify::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

