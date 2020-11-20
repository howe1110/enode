use std::net::{SocketAddr, UdpSocket};

use time::Duration;
use time::PreciseTime;

use crate::message::Message;


pub fn start_client(addr: SocketAddr, dest: SocketAddr) {
    let st = UdpSocket::bind(addr).expect("couldn't bind to address");

    let mut start = PreciseTime::now();
    let mut rate = 0;
    let message = Message::create_man_message(dest, "hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.");

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
}

#[cfg(test)]
mod client {
    use crate::node_shell::NodeShell;
    const SERVER_ADDR1: &'static str = "127.0.0.1:30022";
    const SERVER_ADDR2: &'static str = "127.0.0.1:30023";

    const TESTSTRING:&'static str = "hello world.";
    #[test]
    fn send_1_k_message() {
        let mut node_1 = NodeShell::new(2, SERVER_ADDR1.parse().unwrap());

        for _ in 0..1024000 {
            node_1.send_message(TESTSTRING, SERVER_ADDR2);
        }

        node_1.polling();
    }
}
