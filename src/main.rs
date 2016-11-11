extern crate sysfs_gpio;
extern crate serial;
extern crate toml;
#[macro_use]
extern crate log;
extern crate env_logger;

mod raspi;
mod config;
mod logging;
mod server;
mod connection;

use std::thread;

fn main() {
    logging::init();

    //let mut generator = raspi::Generator::new();
    //generator.send("Hello World!");

    let mut server = server::Server::new();
    let res = thread::spawn(move || server.run());


    res.join();
    info!("Starting RustPager");
    println!("Hello, world!");
}
