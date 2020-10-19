use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Write};
use std::net::SocketAddr;

extern crate ppgw;
use ppgw::node_shell::NodeShell;

fn send2peer(paras: &[&str], loc: &mut NodeShell) -> bool {
    if paras.len() != 1 {
        return false;
    }

    if let Ok(peer) = paras[0].parse::<SocketAddr>() {
        println!("{:?}", peer);
    }

    loc.peer = Some(String::from(paras[0]));
    true
}

fn init_user_fn() -> HashMap<String, fn(&[&str], &mut NodeShell) -> bool> {
    let mut userfn = HashMap::new();
    userfn.insert(
        String::from("sw"),
        send2peer as fn(&[&str], &mut NodeShell) -> bool,
    );
    userfn
}

fn main() {
    println!("Hello, world!");

    let userfns = init_user_fn();
    let port_number = rand::thread_rng().gen_range(30000, 40000);

    let addr = format!("{}:{}", "127.0.0.1", port_number);
    println!("Server start on {}.", addr);

    let mut nodeshell = NodeShell::new(2, addr);

    loop {
        let mut input = String::new();

        print!("==={}>", nodeshell.peer.as_ref().unwrap_or(&String::from("")));
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        if let Ordering::Equal = input.cmp(&"quit".to_string()) {
            break;
        }
        let paras: Vec<&str> = input.split_whitespace().collect();

        if paras.len() == 0 {
            continue;
        }

        if let Some(pfn) = userfns.get(paras[0]) {
            println!("user function is {}", paras[0]);
            pfn(&paras[1..paras.len()], &mut nodeshell);
            continue;
        }

        nodeshell.sendmessage(input);
    }
}
