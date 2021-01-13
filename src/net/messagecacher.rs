use sha2::{Digest, Sha256};
use std::collections::HashSet;
use time::PreciseTime;

use crate::net::Message;
use crate::net::DATAEOF;

pub const MAXMESSAGELEN: usize = 65536;

pub static mut MSGNO: u32 = 0;

pub fn new_message_no() -> u32 {
    unsafe { MSGNO = MSGNO + 1 };
    unsafe { MSGNO }
}

#[derive(Clone)]
pub struct MessageCacher {
    pub msgno: u32,
    pub complete: bool,
    pub start: PreciseTime,
    pub data: Box<Vec<u8>>,
    pub offset: usize, //最小偏移
    pub eof: usize,    //终点值
    pub seen: usize,   //已接收的总值
    pub cache: HashSet<usize>,
}

impl MessageCacher {
    pub fn new() -> MessageCacher {
        let cache = HashSet::new();
        let mut data = Box::new(Vec::with_capacity(MAXMESSAGELEN));
        data.resize(MAXMESSAGELEN, 0);
        MessageCacher {
            msgno: 0,
            complete: true,
            start: PreciseTime::now(),
            data,
            offset: 0,
            eof: 0,
            seen: 0,
            cache,
        }
    }

    pub fn get_buf(&mut self, offset: usize) -> &mut [u8] {
        &mut self.data[offset..]
    }

    pub fn get_data(&self, offset: usize, eof: usize) -> &[u8] {
        &self.data[offset..eof]
    }

    pub fn end_write(&mut self, offset: usize, size: usize, flag: u16) {
        if self.cache.contains(&offset) {
            return;
        }

        self.cache.insert(offset);
        self.seen += size;

        if flag == DATAEOF {
            self.eof = offset + size;
        }

        if offset == self.offset {
            self.offset += size;
        }

        if self.eof == self.seen {
            self.complete = true;
            return;
        }
    }

    pub fn sha256(&self) -> String {
        let data: &[u8] = &self.data[0..];
        let mut hasher = Sha256::default();
        hasher.input(&data);
        format!("{:x}", hasher.result())
    }

    pub fn gaps(&self) -> usize {
        self.offset
    }

    pub fn get_message(&self) -> Message {
        let message: Message = bincode::deserialize(&self.data[0..self.seen]).unwrap();
        message
    }

    pub fn get_offsets(&self) -> &HashSet<usize> {
        &self.cache
    }

    pub fn get_mut_offsets(&mut self) -> &mut HashSet<usize> {
        &mut self.cache
    }

    pub fn reset(&mut self) {
        self.msgno = 0;
        self.complete = true;
        self.start = PreciseTime::now();
        self.offset = 0;
        self.eof = 0;
        self.seen = 0;
        self.cache.clear();
    }

    pub fn new_message(&mut self, msgno: u32) {
        self.reset();
        self.msgno = msgno;
        self.complete = false;
    }
}
