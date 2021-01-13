extern crate ppgw;
use crate::ppgw::app::MessageProcSystem;
use crate::ppgw::framework::Application;
use crate::ppgw::net::NetSystem;

const SERVER_ADDR2: &'static str = "127.0.0.1:30023";

fn main() {
    println!("Hello, {}!", SERVER_ADDR2);
    let app = Application::new();
    app.with(Box::new(NetSystem::new(SERVER_ADDR2.parse().unwrap())))
        .with(Box::new(MessageProcSystem::new(2)))
        .run();
}
