use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::Arc;
use std::sync::Mutex;

use crate::framework::{Notify, System, Worker, World};
use crate::net::NodeEvent;

pub struct MessageProcSystem {
    pub workers: Vec<Worker>,
    sender: Option<SyncSender<NodeEvent>>,
    receiver: Option<Arc<Mutex<Receiver<Notify>>>>,
}

impl MessageProcSystem {
    pub fn new(size: usize) -> Self {
        let workers = Vec::with_capacity(size);
        MessageProcSystem {
            workers,
            sender: None,
            receiver: None,
        }
    }
}

impl System for MessageProcSystem {
    fn setup(&mut self, world: &mut World) {
        let sender = world.remove::<SyncSender<NodeEvent>>().unwrap().clone();
        world.insert(sender.clone()); //还回去
        self.sender = Some(sender);

        //
        let receiver = world
            .remove::<Arc<Mutex<Receiver<Notify>>>>()
            .unwrap()
            .clone();
        world.insert(receiver.clone()); //还回去
        self.receiver = Some(receiver);
    }

    fn run(&mut self) {
        println!("App starting...");
        for id in 0..self.workers.capacity() {
            println!("create worker {}", id);
            self.workers.push(Worker::new(
                id,
                self.sender.as_ref().unwrap().clone(),
                self.receiver.as_ref().unwrap().clone(),
            ));
        }
    }

    fn dispose(&mut self) {

    }
}
