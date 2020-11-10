use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::command_handler::{CommandHandler,CommandType};
use crate::message::MessagePtr;
use crate::node::NodeEvent;

pub enum Source {
    Local,
    Peer,
}

pub enum Notify {
    NewJob {
        y: MessagePtr,
    },
    Command(CommandType),
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
            let mut commander_handler = CommandHandler::new(id, sender);
            //
            loop {
                let notify = receiver.lock().unwrap().recv().unwrap();
                match notify {
                    Notify::NewJob {y} => {
                        commander_handler.handle(y);
                    }
                    Notify::Command(mut cmd) => {cmd.exec();},
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
