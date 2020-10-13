use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Cursor;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::result::Result;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use time::Duration;
use time::PreciseTime;

use crate::message::Message;
use crate::messagecacher::{new_message_no, MessageCacher};
use crate::packet::{ACK, DATA, DATAEOF, MAXPACKETLEN};
use crate::stats::Stats;
use crate::worker::{Notify, Worker};

const WAITTIME: i64 = 1;

pub enum SendError {
    Block,
}

pub struct Node {
    socket: UdpSocket,
    sender: Arc<Mutex<mpsc::Sender<Notify>>>,
    pub workers: Vec<Worker>,
    send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
    recv_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
    stats: Arc<Mutex<Stats>>,
}

impl Node {
    pub fn new<A: ToSocketAddrs>(size: usize, addr: A) -> Node {
        let st = UdpSocket::bind(addr).expect("couldn't bind to address");

        let (sender, receiver) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..workers.capacity() {
            println!("create worker {}", id);
            let cst = st.try_clone().unwrap();
            workers.push(Worker::new(id, cst, Arc::clone(&receiver)));
        }

        let send_cache = Arc::new(Mutex::new(HashMap::new()));
        let recv_cache = Arc::new(Mutex::new(HashMap::new()));
        let stats = Arc::new(Mutex::new(Stats::new()));

        Node {
            socket: st,
            sender,
            workers,
            send_cache,
            recv_cache,
            stats,
        }
    }

    fn on_receive<R: BufRead + Seek>(
        &self,
        src_addr: SocketAddr,
        reader: &mut R,
        sender: mpsc::Sender<Notify>,
    ) {
        let flag = reader.read_u16::<BigEndian>().unwrap();
        match flag >> 15 {
            0 => self.handle_data_packet(src_addr, flag, reader, sender),
            1 => self.handle_ctrl_packet(src_addr, flag, reader),
            _ => (),
        }
    }

    fn handle_data_packet<R: BufRead + Seek>(
        &self,
        src_addr: SocketAddr,
        flag: u16,
        reader: &mut R,
        sender: mpsc::Sender<Notify>,
    ) {
        let len = reader.read_u16::<BigEndian>().unwrap();
        let msgno = reader.read_u32::<BigEndian>().unwrap();
        let offset = reader.read_u32::<BigEndian>().unwrap();
        let ts = reader.read_u32::<BigEndian>().unwrap();
        let mut recv_cache = self.recv_cache.lock().unwrap();
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
        self.send_resp(src_addr, &message);
        if let Some((_, v)) = recv_cache.remove_entry(&src_addr) {
            sender
                .send(Notify::NewJob {
                    x: src_addr,
                    y: v.get_message(),
                })
                .unwrap();
        }
        let mut stats = self.stats.lock().unwrap();
        stats.receive_increase(src_addr);
    }

    fn handle_ctrl_packet<R: BufRead + Seek>(
        &self,
        src_addr: SocketAddr,
        flag: u16,
        reader: &mut R,
    ) {
        match flag {
            ACK => self.handle_data_ack(src_addr, flag, reader),
            _ => (),
        }
    }

    fn handle_data_ack<R: BufRead + Seek>(&self, src_addr: SocketAddr, flag: u16, reader: &mut R) {
        let mut send_cache = self.send_cache.lock().unwrap();
        if let Some(message) = send_cache.get_mut(&src_addr) {
            let msgno = reader.read_u32::<BigEndian>().unwrap(); //消息号
            let msgstate = reader.read_u8().unwrap(); //接收状态
            println!("ack msgno {}, msgstate {}", msgno, msgstate);
            match msgstate {
                0 => {
                    send_cache.remove_entry(&src_addr);
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
            println!("addr {:?} not found.", src_addr);
        }
    }

    fn send_resp(&self, addr: SocketAddr, message: &MessageCacher) {
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
        self.socket.send_to(&vec[0..vec.len()], addr).unwrap();
    }

    fn send_active(&self, addr: SocketAddr, message: &MessageCacher) {
        let mut eof = 0;
        let mut flags = DATA;
        for offset in message.get_offsets().iter() {
            if MAXPACKETLEN >= (message.eof - offset) {
                eof = message.eof;
                flags = DATAEOF;
            } else {
                eof = *offset + MAXPACKETLEN;
            }
            self.send_packet_data(
                flags,
                message.msgno,
                *offset,
                0,
                message.get_data(*offset, eof),
                addr.to_string().as_str(),
            );
            println!(
                "Resend packet msgno {}, offset {} .",
                message.msgno, *offset
            );
        }
    }

    pub fn start_receive(&self) {
        let mut buf: [u8; 4096] = [0; 4096];

        let sender = self.sender.lock().unwrap();

        loop {
            let (l, src_addr) = self
                .socket
                .recv_from(&mut buf)
                .expect("Didn't receive data");
            let packet: &[u8] = &buf[0..l];
            self.on_receive(src_addr, &mut Cursor::new(packet), sender.clone());
        }
    }

    pub fn start_check(&self) {
        loop {
            thread::sleep(std::time::Duration::from_millis(10));
            //请求重发
            let mut send_cache = self.send_cache.lock().unwrap();
            for (k, v) in send_cache.iter_mut() {
                if v.start.to(PreciseTime::now()) > Duration::seconds(WAITTIME + 1) {
                    self.send_active(*k, v);
                    v.start = PreciseTime::now();
                }
            }
            //
            let mut recv_cache = self.recv_cache.lock().unwrap();
            for (k, v) in recv_cache.iter_mut() {
                if v.start.to(PreciseTime::now()) > Duration::seconds(WAITTIME) {
                    self.send_resp(*k, v);
                    v.start = PreciseTime::now();
                }
            }
        }
    }

    fn stop(&mut self) {
        println!("Sending terminate message to all workers.");

        let sender = self.sender.lock().unwrap();
        for _ in &mut self.workers {
            sender.send(Notify::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    fn send_packet_data(
        &self,
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
        writer.write_u32::<BigEndian>(msgno).unwrap(); //msgno
        writer.write_u32::<BigEndian>(offset as u32).unwrap(); //offset
        writer.write_u32::<BigEndian>(ts).unwrap(); //ts

        writer.write(data).unwrap();
        let vec = writer.into_inner();

        self.socket.send_to(&vec[0..vec.len()], addr);
    }

    pub fn send_usermessage(&self, message: &Message, addr: &str) {
        let mut try_times = 100;
        while try_times > 0 {
            try_times -= 1;
            if let Err(SendError::Block) = self.send_message(message, addr) {
                thread::sleep(std::time::Duration::from_millis(0));
            } else {
                break;
            }
        }
        return;
    }

    fn send_message(&self, message: &Message, addr: &str) -> Result<(), SendError> {
        let mut flags = DATA;
        let data: Vec<u8> = bincode::serialize(message).unwrap();
        let mut offset: usize = 0;
        let len: usize = data.len();
        let mut eof = 0;

        let socket_addr = addr.parse::<SocketAddr>().unwrap();

        let mut send_cache = self.send_cache.lock().unwrap();

        if send_cache.contains_key(&socket_addr) {
            return Err(SendError::Block);
        }

        let cache = send_cache
            .entry(socket_addr)
            .or_insert(MessageCacher::new(new_message_no()));
        let message_number = cache.msgno;

        let offsets = cache.get_mut_offsets();
        loop {
            if MAXPACKETLEN >= (len - eof) {
                eof = len;
                flags = DATAEOF;
            } else {
                eof += MAXPACKETLEN;
            }
            self.send_packet_data(flags, message_number, offset, 0, &data[offset..eof], addr);
            offsets.insert(offset);
            offset = eof;
            if offset == len {
                break;
            }
        }
        cache.eof = eof;

        let mut stats = self.stats.lock().unwrap();
        stats.send_increase(addr.parse().unwrap());

        println!("send message msgno {}.", message_number);
        Ok(())
    }

    pub fn send_data<R: BufRead + Seek>(
        &self,
        reader: &mut R,
        addr: &str,
        send_cache: Arc<Mutex<HashMap<SocketAddr, MessageCacher>>>,
    ) {
    }

    pub fn receive_count(&self, addr: &str) -> usize {
        return self
            .stats
            .lock()
            .unwrap()
            .receive_count(addr.parse().unwrap());
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

    #[test]
    fn resend_works() {}
}
