use byteorder::{BigEndian, WriteBytesExt};
use std::io::Cursor;
use std::net::{SocketAddr, UdpSocket};

use time::Duration;
use time::PreciseTime;

pub fn start_server(addr: SocketAddr) {
    let st = UdpSocket::bind(addr).expect("couldn't bind to address");
    let mut buf: [u8; 4096] = [0; 4096];

    let mut start = PreciseTime::now();
    let mut rate = 0;

    loop {
        let result = st.recv_from(&mut buf);
        match result {
            Ok((_, src_addr)) => {
                //
                let buf = vec![];
                let mut writer = Cursor::new(buf); //
                writer.write_u16::<BigEndian>(0x8001).unwrap(); //flags
                writer.write_u32::<BigEndian>(0).unwrap(); //msgno
                writer.write_u8(0).unwrap(); //flags--已接收完成
                let vec = writer.into_inner();
                st.send_to(&vec[0..vec.len()], src_addr).unwrap();
                rate += 1;
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    println!("Encountered an error receiving data: {:?}", e);
                }
            }
        }
        if start.to(PreciseTime::now()) > Duration::seconds(5) {
            println!("receive rate: {} .", rate / 5);
            rate = 0;
            start = PreciseTime::now();
        }
    }
}
