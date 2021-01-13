use crate::app::*;
use crate::framework::*;
use crate::net::*;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::SyncSender;
use std::thread;

/* use time::Duration;
use time::PreciseTime;


pub fn start_client(addr: SocketAddr, dest: SocketAddr) {
    let st = UdpSocket::bind(addr).expect("couldn't bind to address");

    let mut start = PreciseTime::now();
    let mut rate = 0;
    let message = Message::create_man_message(0,1,"hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.");

    loop {
        let data: Vec<u8> = bincode::serialize(&message).unwrap();
        st.send_to(&data[0..], dest).unwrap();
        //
        rate += 1;
        if start.to(PreciseTime::now()) > Duration::seconds(5) {
            println!("send rate: {} .", rate / 5);
            rate = 0;
            start = PreciseTime::now();
        }
    }
} */

const TESTSTRING: &'static str = "hello world.";
const SERVER_ADDR2: &'static str = "127.0.0.1:30023";

pub struct TestSendSystem {
    sender: Option<SyncSender<NodeEvent>>,
}

impl TestSendSystem {
    pub fn new() -> Self {
        TestSendSystem { sender: None }
    }
    pub fn send_message(sender: &SyncSender<NodeEvent>, message: &str, addr: &str) {
        let message = TestSendSystem::create_man_message(0, 1, message);
        let message = EMessage::construct_emessage(addr.parse().unwrap(), message);
        sender.send(NodeEvent::Message(Box::new(message))).unwrap();
    }

    pub fn send_100k_message(sender: &SyncSender<NodeEvent>, message: &str, addr: &str) {
        for _ in 0..102400 {
            TestSendSystem::send_message(sender, message, addr);
        }
    }

    pub fn create_man_message(sendid: NODEIDTYPE, recvid: NODEIDTYPE, message: &str) -> Message {
        Message {
            sendid,
            recvid,
            taskid: 0,
            mstype: SHOWMESSAGE,
            subtype: SHOWMESSAGE,
            msglen: message.len() as u16,
            msgdata: message.as_bytes().to_vec(),
        }
    }
}

impl System for TestSendSystem {
    fn setup(&mut self, world: &mut World) {
        let sender = world.remove::<SyncSender<NodeEvent>>().unwrap().clone();
        world.insert(sender.clone()); //还回去
        self.sender = Some(sender);
        //
    }

    fn run(&mut self) {
        if let Some(sender) = self.sender.take() {
            let handle = std::thread::spawn(move || {
                loop {
                    TestSendSystem::send_100k_message(&sender, TESTSTRING, SERVER_ADDR2);
                }
            });
        }

        println!("App starting...");
    }

    fn dispose(&mut self) {}
}

#[cfg(test)]
mod client {
    use super::*;
    use crate::framework::Application;
    use crate::net::NetSystem;

    const SERVER_ADDR1: &'static str = "127.0.0.1:30022";
    #[test]
    fn send_message() {
        println!("Hello, {}!", SERVER_ADDR1);
        let app = Application::new();
        app.with(Box::new(NetSystem::new(SERVER_ADDR1.parse().unwrap())))
            .with(Box::new(TestSendSystem::new()))
            .run();
    }
}
