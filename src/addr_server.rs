use std::collections::HashMap;
use std::net::SocketAddr;

use serde::{Serialize, Deserialize};

use crate::app;
use crate::command::Command;
use crate::data::{ADDRMESSAGE, NODEIDTYPE};
use crate::emessage::{EMessage, EMessagePtr};
use crate::message::{Message, MSGTYPE};

const PUT_ME: MSGTYPE = 0;
const PUT_OTH: MSGTYPE = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct Addr {
    id: NODEIDTYPE,
    addr: SocketAddr,
}

pub fn new_message_my() -> Message {
    Message {
        sendid: app::get_node_id(),
        recvid: 0,
        taskid: 0,
        mstype: ADDRMESSAGE,
        subtype: PUT_ME,
        msglen: 0,
        msgdata: Vec::new(),
    }
}

pub struct AddrServer<F>
where
    F: FnMut(EMessagePtr) + Send + 'static,
{
    addrs: HashMap<NODEIDTYPE, SocketAddr>,
    send: F,
}

impl<F> AddrServer<F>
where
    F: FnMut(EMessagePtr) + Send + 'static,
{
    pub fn new(send: F) -> Self {
        let addrs = HashMap::new();
        AddrServer { addrs, send }
    }

    pub fn new_message(&self, sendid: NODEIDTYPE, recvid: NODEIDTYPE, addr: &Addr) -> Message {
        let data = bincode::serialize(addr).unwrap();

        Message {
            sendid,
            recvid,
            taskid: 0,
            mstype: ADDRMESSAGE,
            subtype: PUT_OTH,
            msglen: data.len() as u16,
            msgdata: data,
        }
    }
}

impl<F> Command for AddrServer<F>
where
    F: FnMut(EMessagePtr) + Send + 'static,
{
    fn exec(&mut self, message: EMessagePtr) {
        let addr;

        match message.payload.subtype {
            PUT_OTH => {
                let val: Addr = bincode::deserialize(&message.payload.msgdata).unwrap();
                addr = val.addr;
            }
            _ => {
                addr = message.addr;
            }
        }
        
        println!("Receive addr notify from {}", message.addr);

        self.addrs.insert(message.payload.sendid, addr);
        let addr = Addr {
            id: message.payload.sendid,
            addr: message.addr,
        };

        for (k, v) in self.addrs.iter() {
            if *k == message.payload.sendid {
                continue;
            }
            let message = self.new_message(0, 1, &addr);
            let message = EMessage::construct_emessage(*v, message);
            (self.send)(Box::new(message));
        }
    }

    fn get_msgtype(&self) -> MSGTYPE {
        ADDRMESSAGE
    }
}
