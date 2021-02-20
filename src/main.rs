const VERSION: &str = "0.0.1";

extern crate dotenv;
use dotenv::dotenv;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate config;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod client;
mod handlers;
mod server;
mod settings;
use std::time::Duration;

use server::Server;

fn main() {
    dotenv().ok();
    pretty_env_logger::init_timed();
    print_splash();
    let mut server = Server::new().unwrap();
    loop {
        server.tick().unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn print_splash() {
    const SPLASH: &str = r#"
 ____
|  _ \ ___  ___  ___  ___ _ ____   __
| |_) / _ \/ _ \/ __|/ _ | '__\ \ / /
|  _ |  __| (_) \__ |  __| |   \ V /
|_| \_\___|\___/|___/\___|_|    \_/
The Rusty Endless Online Server Emulator
"#;
    println!("{}Version: {}", SPLASH, VERSION);
}
