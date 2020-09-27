use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

use crate::message::Message;

pub enum Notify {
    NewJob { x: SocketAddr, y: Message },
    Terminate,
}

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, st: UdpSocket, receiver: Arc<Mutex<mpsc::Receiver<Notify>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let notify = receiver.lock().unwrap().recv().unwrap();

            match notify {
                Notify::NewJob { x, y } => {
                    println!("Worker {} got a job; executing.", id);
                    Worker::handle_message(x, &y);
                }
                Notify::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }

    fn handle_message(x: SocketAddr, y: &Message) {
        println!("handle message {:?} from {:?}", y, x);
    }
}
