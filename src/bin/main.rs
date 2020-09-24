use ppgw::ENode;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use rand::Rng;

fn send2peer(paras: &[&str], loc: &ENode) -> bool {
    if paras.len() != 2 {
        return false;
    }
    
    println!("{}:{}", paras[0], paras[1]);

    if let Ok(peer) = paras[0].parse::<SocketAddr>() {
        println!("{:?}", peer);
    }
    true
}

fn init_user_fn() -> HashMap<String, fn(&[&str], &ENode) -> bool> {
    let mut userfn = HashMap::new();
    userfn.insert(
        String::from("sendto"),
        send2peer as fn(&[&str], &ENode) -> bool,
    );
    userfn
}

fn main() {
    println!("Hello, world!");

    let userfns = init_user_fn();
    let port_number = rand::thread_rng().gen_range(30000, 40000);

    let addr = format!("{}:{}","127.0.0.1",port_number);
    println!("Server start on {}.", addr);
    let mut server = ENode::new(2, addr);

    server.start();

    loop {
        let mut input = String::new();
        println!("Please input your guess.");

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        if let Ordering::Equal = input.cmp(&"quit".to_string()) {
            break;
        }
        let paras: Vec<&str> = input.split_whitespace().collect();

        println!("user function is {}", paras[0]);

        if let Some(pfn) = userfns.get(paras[0]) {
            pfn(&paras[1..paras.len()], &server);
        }
    }
}
