use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::VecDeque;
use std::io::prelude::*;
use std::io::Cursor;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self};

use time::Duration;
use time::PreciseTime;

use crate::emessage::{EMessage, EMessagePtr};
use crate::message::MessagePtr;
use crate::messagecacher::{new_message_no, MessageCacher};
use crate::packet::{ACK, DATA, DATAEOF, MAXPACKETLEN};
use crate::stats::Stats;
use crate::worker::Notify;

const WAITTIME: i64 = 1;

pub enum SendError {
    Block,
}

pub enum TrySendResult {
    Wait,
    Empty,
    Ok,
}

pub struct Connection {
    pub buffer: VecDeque<EMessagePtr>,
    pub socket: UdpSocket,
    pub send_cache: MessageCacher,
    pub recv_cache: MessageCacher,
    pub addr: SocketAddr,
    pub inner_sender: mpsc::Sender<Notify>,
    pub stats: Stats,
}

impl Connection {
    pub fn new(
        socket: UdpSocket,
        addr: SocketAddr,
        inner_sender: mpsc::Sender<Notify>,
    ) -> Connection {
        let send_cache = MessageCacher::new();
        let recv_cache = MessageCacher::new();
        let stats = Stats::new();
        let buffer = VecDeque::with_capacity(2048);
        Connection {
            buffer,
            socket,
            send_cache,
            recv_cache,
            addr,
            inner_sender,
            stats,
        }
    }

    pub fn push_message(&mut self, message: EMessagePtr) {
        self.buffer.push_front(message);
    }

    pub fn on_receive<R: BufRead + Seek>(&mut self, reader: &mut R) {
        let flag = reader.read_u16::<BigEndian>().unwrap();
        match flag >> 15 {
            0 => self.handle_data_packet(flag, reader),
            1 => self.handle_ctrl_packet(flag, reader),
            _ => (),
        }
    }

    pub fn send(&mut self) -> TrySendResult {
        if !self.send_cache.complete {
            return TrySendResult::Ok;
        }

        if let Some(message) = self.buffer.pop_back() {
            self.send_message(message.payload);
            return TrySendResult::Ok;
        }
        TrySendResult::Empty
    }

    pub fn start_check(&mut self) {
        //请求重发
        if !self.send_cache.complete {
            if self.send_cache.start.to(PreciseTime::now()) > Duration::seconds(WAITTIME + 1) {
                self.send_active();
                self.send_cache.start = PreciseTime::now();
            }
        }
        //
        if !self.recv_cache.complete {
            if self.recv_cache.start.to(PreciseTime::now()) > Duration::seconds(WAITTIME) {
                self.send_resp();
                self.recv_cache.start = PreciseTime::now();
            }
        }
        //
        if self.stats.start.to(PreciseTime::now()) > Duration::seconds(5) {
            self.stats.show(5);
        }
    }

    fn handle_data_packet<R: BufRead + Seek>(&mut self, flag: u16, reader: &mut R) {
        let len = reader.read_u16::<BigEndian>().unwrap();
        let msgno = reader.read_u32::<BigEndian>().unwrap();
        let offset = reader.read_u32::<BigEndian>().unwrap();
        let _ts = reader.read_u32::<BigEndian>().unwrap();

        if msgno != self.recv_cache.msgno {
            self.recv_cache.new_message(msgno);
        }
        //
        let _ = reader
            .take(len as u64)
            .read(&mut self.recv_cache.get_buf(offset as usize));
        self.recv_cache
            .end_write(offset as usize, len as usize, flag);

        if self.recv_cache.complete == false {
            return;
        }

        self.recv_cache.start = PreciseTime::now();

        self.inner_sender
            .send(Notify::NewJob {
                y: Box::new(EMessage::construct_emessage(
                    self.addr,
                    self.recv_cache.get_message(),
                )),
            })
            .unwrap();
        //
        self.stats.receive_increase();
        self.send_resp();
    }

    fn handle_ctrl_packet<R: BufRead + Seek>(&mut self, flag: u16, reader: &mut R) {
        match flag {
            ACK => self.handle_data_ack(flag, reader),
            _ => (),
        }
    }

    fn handle_data_ack<R: BufRead + Seek>(&mut self, _flag: u16, reader: &mut R) {
        let _msgno = reader.read_u32::<BigEndian>().unwrap(); //消息号
        let msgstate = reader.read_u8().unwrap(); //接收状态
        match msgstate {
            0 => {
                self.send_cache.reset();
            } //接收完成
            1 => {
                let offsetnum = reader.read_u32::<BigEndian>().unwrap(); //分片数
                for _ in 0..offsetnum {
                    let offset = reader.read_u32::<BigEndian>().unwrap();
                    let offset = offset as usize;
                    //分片
                    self.send_cache.cache.remove(&offset);
                }
            }
            _ => (),
        }
    }

    pub fn send_resp(&mut self) {
        let buf = vec![];
        let mut writer = Cursor::new(buf); //
        writer.write_u16::<BigEndian>(ACK).unwrap(); //flags
        writer
            .write_u32::<BigEndian>(self.recv_cache.msgno)
            .unwrap(); //msgno
        if self.recv_cache.complete == true {
            writer.write_u8(0).unwrap(); //flags--已接收完成
        } else {
            writer.write_u8(1).unwrap(); //flags--未接收完成
            writer
                .write_u32::<BigEndian>(self.recv_cache.get_offsets().len() as u32)
                .unwrap(); //分片数
            for offset in self.recv_cache.get_offsets().iter() {
                writer.write_u32::<BigEndian>(*offset as u32).unwrap(); //offset
            }
        }
        let vec = writer.into_inner();
        self.socket.send_to(&vec[0..vec.len()], self.addr).unwrap();
    }

    fn send_active(&mut self) {
        let mut eof;
        let mut flags;

        for offset in self.send_cache.get_offsets().iter() {
            if MAXPACKETLEN >= (self.send_cache.eof - offset) {
                eof = self.send_cache.eof;
                flags = DATAEOF;
            } else {
                eof = *offset + MAXPACKETLEN;
                flags = DATA;
            }
            //
            let buf = vec![];
            let mut writer = Cursor::new(buf);

            let data = self.send_cache.get_data(*offset, eof);

            writer.write_u16::<BigEndian>(flags).unwrap(); //flags
            writer.write_u16::<BigEndian>(data.len() as u16).unwrap(); //length
            writer
                .write_u32::<BigEndian>(self.send_cache.msgno)
                .unwrap(); //msgno
            writer.write_u32::<BigEndian>(*offset as u32).unwrap(); //offset
            writer.write_u32::<BigEndian>(0).unwrap(); //ts
            writer.write(data).unwrap();
            let vec = writer.into_inner();
            let result = self.socket.send_to(&vec[0..vec.len()], &self.addr);
            //
            match result {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }

    pub fn send_message(&mut self, message: MessagePtr) {
        let data: Vec<u8> = bincode::serialize(&message).unwrap();
        let mut offset: usize = 0;
        let len: usize = data.len();
        let mut eof = 0;

        self.send_cache.data = Box::new(data);

        self.send_cache.new_message(new_message_no());

        let offsets = self.send_cache.get_mut_offsets();
        loop {
            if MAXPACKETLEN >= (len - eof) {
                eof = len;
            } else {
                eof += MAXPACKETLEN;
            }
            offsets.insert(offset);
            offset = eof;
            if offset == len {
                break;
            }
        }
        self.send_cache.eof = eof;
        self.send_cache.start = PreciseTime::now();

        self.send_active();

        self.stats.send_increase();
    }
}
