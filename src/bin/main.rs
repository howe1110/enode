extern crate ppgw;
use crate::ppgw::node_shell::NodeShell;

const SERVER_ADDR2: &'static str = "127.0.0.1:30023";

fn main() {
    println!("Hello, {}!", SERVER_ADDR2);
    let mut nodeshell = NodeShell::new(2, SERVER_ADDR2.parse().unwrap());
    nodeshell.polling();
}
