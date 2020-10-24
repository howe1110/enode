use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Cursor;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::rc::Rc;
use std::sync::mpsc::{self, sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, yield_now};

use crate::connection::Connection;
use crate::emessage::EMessage;
use crate::message::Message;
use crate::stats::Stats;
use crate::worker::{Notify, Worker};

const WAITTIME: i64 = 1;

pub enum SendError {
    Block,
}

pub enum ConnectError {
    AllReadyExist,
}

pub struct ConnectEvent {
    addr: SocketAddr,
    receiver: Receiver<Message>,
}

impl ConnectEvent {
    pub fn new(addr: SocketAddr, receiver: Receiver<Message>) -> ConnectEvent {
        ConnectEvent { addr, receiver }
    }
}
pub struct Node {
    socket: UdpSocket,
    sender: Rc<mpsc::Sender<Notify>>,
    pub workers: Vec<Worker>,
    connections: HashMap<SocketAddr, Connection>,
    pub connect_receiver: Receiver<ConnectEvent>,
    stats: Arc<Stats>,
}

impl Node {
    pub fn new<A: ToSocketAddrs>(
        size: usize,
        addr: A,
        connect_receiver: Receiver<ConnectEvent>,
    ) -> Node {
        let st = UdpSocket::bind(addr).expect("couldn't bind to address");
        st.set_nonblocking(true).unwrap();

        let (sender, receiver) = mpsc::channel();
        let sender = Rc::new(sender);
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..workers.capacity() {
            println!("create worker {}", id);
            let cst = st.try_clone().unwrap();
            workers.push(Worker::new(id, cst, Arc::clone(&receiver)));
        }

        let connections = HashMap::new();
        let stats = Arc::new(Stats::new());

        Node {
            socket: st,
            sender,
            workers,
            connections,
            stats,
            connect_receiver,
        }
    }

    pub fn handle_connect(&mut self) {
        let result = self.connect_receiver.try_recv();
        match result {
            Ok(conn_event) => {
                let connect = self
                    .connections
                    .entry(conn_event.addr)
                    .or_insert(Connection::new(
                        self.socket.try_clone().unwrap(),
                        conn_event.addr,
                        self.sender.clone(),
                    ));
                connect.set_receiver(conn_event.receiver);
            }
            Err(e) => {
                if e != TryRecvError::Empty {
                    println!("Connection try_recv error {:?} .", e);
                }
            }
        }
    }

    fn start_receive(&mut self) {
        let mut buf: [u8; 4096] = [0; 4096];
        let result = self.socket.recv_from(&mut buf);

        match result {
            Ok((l, src_addr)) => {
                let connect = self.connections.entry(src_addr).or_insert(Connection::new(
                    self.socket.try_clone().unwrap(),
                    src_addr,
                    self.sender.clone(),
                ));
                //
                let packet: &[u8] = &buf[0..l];
                connect.on_receive(&mut Cursor::new(packet));
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    println!("Encountered an error receiving data: {:?}", e);
                }
            }
        }
    }

    fn start_send(&mut self) {
        for (_, v) in self.connections.iter_mut() {
            v.send();
        }
    }

    pub fn start_polling(&mut self) {
        loop {
            self.start_send();
            self.handle_connect();
            self.start_receive();
            self.start_check();
        }
    }

    fn start_check(&mut self) {
        //请求重发
        for (_, v) in self.connections.iter_mut() {
            v.start_check();
        }
    }

    pub fn stop(&mut self) {
        //
        println!("Sending terminate message to all workers.");
        for _ in &mut self.workers {
            self.sender.send(Notify::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    pub fn receive_count(&self, addr: &SocketAddr) -> usize {
        if let Some(conn) = self.connections.get(addr) {
            return conn.stats.receive_count;
        } else {
            0
        }
    }

    pub fn send_count(&self, addr: &SocketAddr) -> usize {
        if let Some(conn) = self.connections.get(addr) {
            return conn.stats.send_count;
        } else {
            0
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::{self, yield_now};

    const SERVER_ADDR: &'static str = "127.0.0.1:30022";

    fn server_address() -> SocketAddr {
        SERVER_ADDR.parse().unwrap()
    }

    #[test]
    fn send_works() {
        let (sender, receiver) = sync_channel(0);
        let handle = std::thread::spawn(move || {
            let mut node = Node::new(2, "127.0.0.1:30020", receiver);
            node.start_polling();
        });

        let (message_sender, message_receiver) = sync_channel(0);

        let connect = ConnectEvent::new(server_address(), message_receiver);
        sender.send(connect);
        let message = Message::create_man_message("hello world.");
        message_sender.send(message);

        handle.join().unwrap();
    }
}
