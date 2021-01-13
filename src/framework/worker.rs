use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::framework::{CommandHandler};
use crate::net::EMessagePtr;

use crate::net::NodeEvent;

pub enum Source {
    Local,
    Peer,
}

pub enum Notify {
    NewJob { y: EMessagePtr },
    Terminate,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    #[warn(unused_variables)]
    pub fn new(
        id: usize,
        sender: SyncSender<NodeEvent>,
        receiver: Arc<Mutex<mpsc::Receiver<Notify>>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            let mut commander_handler = CommandHandler::new(sender);

            //
            loop {
                let notify = receiver.lock().unwrap().recv().unwrap();
                match notify {
                    Notify::NewJob { y } => {
                        commander_handler.handle(y);
                    }
                    Notify::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
