use std::collections::HashSet;
use time::PreciseTime;
use sha2::{Digest, Sha256};

use crate::message::Message;
use crate::packet::{DATAEOF};

pub const MAXMESSAGELEN: usize = 65536;

pub struct MessageCacher {
    pub msgno:u32,
    pub complete: bool,
    pub start: PreciseTime,
    data: [u8; MAXMESSAGELEN],
    pub offset: usize, //最小偏移
    pub eof: usize,    //终点值
    pub seen: usize,   //已接收的总值
    pub cache:HashSet<usize>,
}

impl MessageCacher {
    pub fn new(msgno:u32) -> MessageCacher {
        let cache = HashSet::new();
        MessageCacher {
            msgno,
            complete: false,
            start: PreciseTime::now(),
            data: [0; MAXMESSAGELEN],
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

        self.cache.insert(offset);
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

    pub fn get_offsets(& self) -> &HashSet<usize> {
        &self.cache
    }

    pub fn get_mut_offsets(&mut self) -> &mut HashSet<usize> {
        &mut self.cache
    }
}
