extern crate bit_field;
extern crate byteorder;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use data::{Message, MessageCacher, ACK, DATA, DATAEOF, MAXPACKETLEN};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Cursor;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use time::Duration;
use time::PreciseTime;
use timer::Guard;

extern crate chrono;
extern crate timer;

use std::sync::mpsc::channel;

pub mod data;

enum Notify {
    NewJob { x: SocketAddr, y: Message },
    Terminate,
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, st: UdpSocket, receiver: Arc<Mutex<mpsc::Receiver<Notify>>>) -> Worker {
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

pub struct ENode {
    socket: UdpSocket,
    sender: mpsc::Sender<Notify>,
    pub workers: Vec<Worker>,
    thread: Option<thread::JoinHandle<()>>,
    thread_check: Option<thread::JoinHandle<()>>,
}

pub struct Exch;

impl ENode {
    pub fn new<A: ToSocketAddrs>(size: usize, addr: A) -> ENode {
        let mut workers = Vec::with_capacity(size);

        let st = UdpSocket::bind(addr).expect("couldn't bind to address");

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            println!("create worker {}", id);
            let cst = st.try_clone().unwrap();
            workers.push(Worker::new(id, cst, Arc::clone(&receiver)));
        }

        ENode {
            socket: st,
            sender,
            workers,
            thread: None,
            thread_check: None,
        }
    }

    pub fn start(&mut self) {
        let socket = self.socket.try_clone().unwrap();
        let socket_check = self.socket.try_clone().unwrap();
        let sender = self.sender.clone();
        let tick_counter_10ms = Arc::new(Mutex::new(0));
        let send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let recv_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>> =
            Arc::new(Mutex::new(HashMap::new()));

        self.start_receive(
            socket,
            sender,
            recv_cache.clone(),
            send_cache.clone(),
            tick_counter_10ms.clone(),
        );
        self.start_check(
            socket_check,
            recv_cache.clone(),
            send_cache.clone(),
            tick_counter_10ms.clone(),
        );
    }

    fn on_receive<R: BufRead + Seek>(
        src_addr: SocketAddr,
        socket: UdpSocket,
        reader: &mut R,
        recv_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        sender: mpsc::Sender<Notify>,
    ) {
        let flag = reader.read_u16::<BigEndian>().unwrap();
        match flag >> 15 {
            0 => ENode::handle_data_packet(src_addr, socket, flag, reader, recv_cache, sender),
            1 => ENode::handle_ctrl_packet(src_addr, socket, flag, reader, send_cache, sender),
            _ => (),
        }
    }

    fn handle_data_packet<R: BufRead + Seek>(
        src_addr: SocketAddr,
        socket: UdpSocket,
        flag: u16,
        reader: &mut R,
        recv_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        sender: mpsc::Sender<Notify>,
    ) {
        let len = reader.read_u16::<BigEndian>().unwrap();
        let msgno = reader.read_u32::<BigEndian>().unwrap();
        let offset = reader.read_u32::<BigEndian>().unwrap();
        let ts = reader.read_u32::<BigEndian>().unwrap();
        let mut recv_cache = recv_cache.lock().unwrap();
        let message = recv_cache
            .entry(src_addr)
            .or_insert(MessageCacher::new(msgno));
        let _ = reader
            .take(len as u64)
            .read(&mut message.get_buf(offset as usize));
        message.end_write(offset as usize, len as usize, flag);

        if message.complete == false {
            return;
        }
        ENode::send_resp(src_addr, socket, &message);
        if let Some((_, v)) = recv_cache.remove_entry(&src_addr) {
            sender
                .send(Notify::NewJob {
                    x: src_addr,
                    y: v.get_message(),
                })
                .unwrap();
        }
    }

    fn handle_ctrl_packet<R: BufRead + Seek>(
        src_addr: SocketAddr,
        socket: UdpSocket,
        flag: u16,
        reader: &mut R,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        sender: mpsc::Sender<Notify>,
    ) {
        println!("Handle ctrl packet.");
        match flag {
            ACK => ENode::handle_data_ack(src_addr, socket, flag, reader, send_cache, sender),
            _ => (),
        }
    }

    fn handle_data_ack<R: BufRead + Seek>(
        src_addr: SocketAddr,
        socket: UdpSocket,
        flag: u16,
        reader: &mut R,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        sender: mpsc::Sender<Notify>,
    ) {
        println!("Handle Ack.");
        let mut send_cache = send_cache.lock().unwrap();
        if let Some(message) = send_cache.get_mut(&src_addr) {
            let msgno = reader.read_u32::<BigEndian>().unwrap(); //消息号
            println!("ack msgno {}", msgno);
            let msgstate = reader.read_u8().unwrap(); //接收状态
            println!("ack msgstate {}", msgstate);
            match msgstate {
                0 => {
                    message.cache.clear();
                } //接收完成
                1 => {
                    let offsetnum = reader.read_u32::<BigEndian>().unwrap(); //分片数
                    for _ in 0..offsetnum {
                        let offset = reader.read_u32::<BigEndian>().unwrap();
                        let offset = offset as usize;
                        //分片
                        message.cache.remove(&offset);
                    }
                }
                _ => (),
            }
        } else {
        }
    }

    fn send_resp(addr: SocketAddr, socket: UdpSocket, message: &MessageCacher) {
        let buf = vec![];
        let mut writer = Cursor::new(buf); //
        writer.write_u16::<BigEndian>(ACK).unwrap(); //flags
        writer.write_u32::<BigEndian>(message.msgno).unwrap(); //msgno
        if message.complete == true {
            writer.write_u8(0).unwrap(); //flags--已接收完成
        } else {
            writer.write_u8(1).unwrap(); //flags--未接收完成
            writer
                .write_u32::<BigEndian>(message.get_offsets().len() as u32)
                .unwrap(); //分片数
            for offset in message.get_offsets().iter() {
                writer.write_u32::<BigEndian>(*offset as u32).unwrap(); //offset
            }
        }
        let vec = writer.into_inner();
        socket.send_to(&vec[0..vec.len()], addr).unwrap();
        println!("Send response.");
    }

    fn send_active(socket: UdpSocket, addr: SocketAddr, message: &MessageCacher) {
        let mut eof = 0;
        let mut flags = DATA;
        for offset in message.get_offsets().iter() {
            if MAXPACKETLEN >= (message.eof - offset) {
                eof = message.eof;
                flags = DATAEOF;
            } else {
                eof = *offset + MAXPACKETLEN;
            }
            ENode::send_packet_data(
                socket.try_clone().unwrap(),
                flags,
                0,
                *offset,
                0,
                message.get_data(*offset, eof),
                addr.to_string().as_str(),
            );
        }
    }

    fn start_receive(
        &mut self,
        socket: UdpSocket,
        sender: mpsc::Sender<Notify>,
        recv_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        ticks_count: Arc<Mutex<usize>>,
    ) {
        let handle = std::thread::spawn(move || {
            let mut buf: [u8; 4096] = [0; 4096];
            loop {
                let (l, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
                let packet: &[u8] = &buf[0..l];
                ENode::on_receive(
                    src_addr,
                    socket.try_clone().unwrap(),
                    &mut Cursor::new(packet),
                    recv_cache.clone(),
                    send_cache.clone(),
                    sender.clone(),
                );
            }
        });
        self.thread = Some(handle);
    }

    fn start_check(
        &mut self,
        socket: UdpSocket,
        recv_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
        ticks_count: Arc<Mutex<usize>>,
    ) {
        let recv_cache = recv_cache.clone();


        let handle = std::thread::spawn(move || {
            loop {
                thread::sleep(std::time::Duration::from_millis(10));

                //请求重发
                let send_cache = send_cache.lock().unwrap();
                for (k, v) in send_cache.iter() {
                    if v.start.to(PreciseTime::now()) > Duration::seconds(30) {
                        ENode::send_active(socket.try_clone().unwrap(), *k, v);
                    }
                }
                //
                let recv_cache = recv_cache.lock().unwrap();
                for (k, v) in recv_cache.iter() {
                    if v.start.to(PreciseTime::now()) > Duration::seconds(30) {
                        ENode::send_resp(*k, socket.try_clone().unwrap(), v);
                    }
                }
            }
        });
        self.thread_check = Some(handle);
    }

    fn stop(&mut self) {
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

    fn send<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> usize {
        self.socket.send_to(&buf, addr).expect("couldn't send data")
    }

    fn send_packet_data(
        socket: UdpSocket,
        flags: u16,
        msgno: u32,
        offset: usize,
        ts: u32,
        data: &[u8],
        addr: &str,
    ) {
        let buf = vec![];
        let mut writer = Cursor::new(buf);
        writer.write_u16::<BigEndian>(flags).unwrap(); //flags
        writer.write_u16::<BigEndian>(data.len() as u16).unwrap(); //length
        println!("write flag {}", flags);
        writer.write_u32::<BigEndian>(msgno).unwrap(); //msgno
        writer.write_u32::<BigEndian>(offset as u32).unwrap(); //offset
        writer.write_u32::<BigEndian>(ts).unwrap(); //ts

        writer.write(data).unwrap();
        let vec = writer.into_inner();

        socket.send_to(&vec[0..vec.len()], addr);
    }

    pub fn send_message(
        socket: UdpSocket,
        message: &Message,
        addr: &str,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
    ) {
        let mut flags = DATA;
        let data: Vec<u8> = bincode::serialize(message).unwrap();
        let mut offset: usize = 0;
        let len: usize = data.len();
        let mut eof = 0;

        let mut send_cache = send_cache.lock().unwrap();
        let cache = send_cache
            .entry(addr.parse().unwrap())
            .or_insert(MessageCacher::new(123));
        let offsets = cache.get_mut_offsets();
        loop {
            if MAXPACKETLEN >= (len - eof) {
                eof = len;
                flags = DATAEOF;
            } else {
                eof += MAXPACKETLEN;
            }
            println!("flag: {}, offset: {}, eof:{}", flags, offset, eof);
            ENode::send_packet_data(
                socket.try_clone().unwrap(),
                flags,
                0,
                offset,
                0,
                &data[offset..eof],
                addr,
            );
            offsets.insert(offset);
            offset = eof;
            if offset == len {
                break;
            }
        }
        cache.eof = eof;
    }

    pub fn send_data<R: BufRead + Seek>(
        &self,
        reader: &mut R,
        addr: &str,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
    ) {
    }
}

impl Drop for ENode {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::time::Duration;

    fn send_message(len: usize, sport: u16, dport: u16) {
        let daddr = format!("{}:{}", "127.0.0.1", sport);
        let saddr = format!("{}:{}", "127.0.0.1", dport);

        let mut data: Vec<u8> = Vec::new();
        for _ in 0..len {
            data.push(0);
        }

        let message = Message {
            mstype: 12,
            msglen: data.len() as u16,
            msgdata: data,
        };

        let mut p1 = ENode::new(2, daddr.clone());
        p1.start();
        //
        let mut p2 = ENode::new(2, saddr);
        p2.start();
        p2.send_message(&message, daddr.as_str());

        thread::sleep(Duration::from_secs(10));
    }

    #[test]
    fn test_send_one_packet_message() {
        send_message(256, 20001, 30001);
    }

    #[test]
    fn test_send_two_packet_message() {
        send_message(2560, 40001, 50001);
    }
}
