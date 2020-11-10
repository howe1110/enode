use std::net::{SocketAddr, UdpSocket};

use time::Duration;
use time::PreciseTime;

use crate::message::Message;

pub fn start_client(addr: SocketAddr, dest: SocketAddr) {
    let st = UdpSocket::bind(addr).expect("couldn't bind to address");

    let mut start = PreciseTime::now();
    let mut rate = 0;
    let message = Message::create_man_message("hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.hello world.");

    loop {
        let data: Vec<u8> = bincode::serialize(&message).unwrap();
        st.send_to(&data[0..], dest);
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
mod tests {
    use super::*;

    const SERVER_ADDR11: &'static str = "127.0.0.1:30022";
    const SERVER_ADDR22: &'static str = "127.0.0.1:30023";

    fn server_address1() -> SocketAddr {
        SERVER_ADDR11.parse().unwrap()
    }

    fn server_address2() -> SocketAddr {
        SERVER_ADDR22.parse().unwrap()
    }

    #[test]
    fn client_send_works() {
        start_client(server_address1(), server_address2());
    }
}
