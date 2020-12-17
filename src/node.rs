use std::collections::HashMap;
use std::io::Cursor;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::connection::Connection;
use crate::connection::TrySendResult;
use crate::emessage::EMessagePtr;
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
    Message(EMessagePtr),
    Terminate,
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
        let mut buf: [u8; 4096] = [0; 4096];
        let mut block_count = 0;
        loop {
            let mut idle = true;
            let result = self.connect_receiver.try_recv();
            match result {
                Ok(e) => match e {
                    NodeEvent::Message(message) => {
                        self.handle_message_event(message);
                        idle = false;
                    }
                    NodeEvent::Terminate => break,
                },
                Err(_) => (),
            }
            //接收消息
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
                    idle = false;
                    block_count = 1000;//优化CPU使用，有消息时继续循环处理，防止因为暂时收不到消息而进入休眠。
                }
                Err(_) => {
                    if block_count > 0 {
                        block_count -= 1;
                        idle = false;
                    }
                }
            }
            //发送消息
            for (_, v) in self.connections.iter_mut() {
                match v.send() {
                    TrySendResult::Ok => {
                        idle = false;
                    }
                    _ => (),
                }
            }

            self.start_check();

            if idle {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    fn handle_message_event(&mut self, message: EMessagePtr) {
        if let Some(conn) = self.connections.get_mut(&message.addr) {
            conn.push_message(message);
            return;
        }
        let mut conn = Connection::new(
            self.socket.try_clone().unwrap(),
            message.addr,
            self.inner_sender.clone(),
        );
        let addr = message.addr;
        conn.push_message(message);
        self.connections.insert(addr, conn);
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
