use std::collections::HashMap;
use std::io::Cursor;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::yield_now;

use crate::connection::Connection;
use crate::connection::TrySendResult;
use crate::message::MessagePtr;
use crate::worker::Notify;

pub enum SendError {
    Block,
}

pub enum ConnectError {
    AllReadyExist,
}

pub enum HandleNodeEventResult {
    Block,
    Exit,
}

pub enum NodeReceiveResult {
    Ok,
    Block,
}

pub enum NodeEvent {
    Connect(ConnectEvent),
    Terminate,
}

pub struct ConnectEvent {
    id: usize,
    addr: SocketAddr,
    receiver: Receiver<MessagePtr>,
}

pub struct UserMessageEvent {
    id: usize,
    addr: SocketAddr,
}

impl ConnectEvent {
    pub fn new(id: usize, addr: SocketAddr, receiver: Receiver<MessagePtr>) -> ConnectEvent {
        ConnectEvent { id, addr, receiver }
    }
}
pub struct Node {
    socket: UdpSocket,
    inner_sender: mpsc::Sender<Notify>,
    connections: HashMap<SocketAddr, Connection>,
    pub connect_receiver: Receiver<NodeEvent>,
}

impl Node {
    pub fn new(
        addr: SocketAddr,
        connect_receiver: Receiver<NodeEvent>,
        inner_sender: Sender<Notify>,
    ) -> Node {
        let st = UdpSocket::bind(addr).expect("couldn't bind to address");
        st.set_nonblocking(true).unwrap();

        let connections = HashMap::new();

        Node {
            socket: st,
            inner_sender,
            connections,
            connect_receiver,
        }
    }

    pub fn start_polling(&mut self) {
        let mut node_event_idle = false;
        let mut node_send_idle = false;
        let mut node_receive_idle = false;

        loop {
            if let Err(e) = self.handle_node_event() {
                match e {
                    HandleNodeEventResult::Exit => {
                        println!("Node Exit.");
                        break;
                    }
                    HandleNodeEventResult::Block => {
                        node_event_idle = true;
                    }
                }
            }
            match self.start_receive() {
                NodeReceiveResult::Block => {
                    node_receive_idle = true;
                }
                _ => (),
            }
            //
            node_send_idle = self.start_send();

            if node_event_idle && node_send_idle && node_receive_idle {
                self.start_check();
                yield_now();
            }
        }
    }

    fn handle_conn_event(&mut self, id: usize, addr: SocketAddr, receiver: Receiver<MessagePtr>) {
        let connect = self.connections.entry(addr).or_insert(Connection::new(
            self.socket.try_clone().unwrap(),
            addr,
            self.inner_sender.clone(),
        ));
        connect.set_receiver(id, receiver);
    }

    fn handle_node_event(&mut self) -> Result<(), HandleNodeEventResult> {
        let result = self.connect_receiver.try_recv();
        match result {
            Ok(e) => match e {
                NodeEvent::Connect(conn) => {
                    self.handle_conn_event(conn.id, conn.addr, conn.receiver);
                    Ok(())
                }
                NodeEvent::Terminate => Err(HandleNodeEventResult::Exit),
            },
            Err(e) => {
                if e != TryRecvError::Empty {
                    println!("Connection try_recv error {:?} .", e);
                }
                Err(HandleNodeEventResult::Block)
            }
        }
    }

    fn start_receive(&mut self) -> NodeReceiveResult {
        let mut buf: [u8; 4096] = [0; 4096];
        let result = self.socket.recv_from(&mut buf);

        match result {
            Ok((l, src_addr)) => {
                let packet: &[u8] = &buf[0..l];
                if let Some(conn) = self.connections.get_mut(&src_addr) {
                    conn.on_receive(&mut Cursor::new(packet));
                } else {
                    let mut conn = Connection::new(
                        self.socket.try_clone().unwrap(),
                        src_addr,
                        self.inner_sender.clone(),
                    );
                    conn.on_receive(&mut Cursor::new(packet));
                    self.connections.insert(src_addr, conn);
                }
                NodeReceiveResult::Ok
            }
            Err(_) => NodeReceiveResult::Block,
        }
    }

    fn start_send(&mut self) -> bool {
        let mut is_idle = false;
        for (_, v) in self.connections.iter_mut() {
            match v.send() {
                TrySendResult::Empty => {
                    is_idle = true;
                }
                _ => (),
            }
        }
        is_idle
    }

    fn start_check(&mut self) {
        //请求重发
        for (_, v) in self.connections.iter_mut() {
            v.start_check();
        }
    }

    pub fn stop(&mut self) {}

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
    fn send_works() {}
}
