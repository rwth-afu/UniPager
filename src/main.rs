extern crate sysfs_gpio;
extern crate serial;
extern crate toml;
#[macro_use]
extern crate log;
extern crate env_logger;

mod c9000;
mod config;
mod logging;
mod server;
mod connection;
mod message;
mod scheduler;

use std::thread;

fn main() {
    logging::init();

    //let mut generator = raspi::Generator::new();
    //generator.send("Hello World!");

    info!("Starting RustPager");

    let config = config::Config::load();

    let scheduler = scheduler::Scheduler::new();

    let scheduler1 = scheduler.clone();
    thread::spawn(move || scheduler1.run());

    let server = server::Server::new(&config);
    let res = thread::spawn(move || server.run(scheduler));

    res.join().unwrap();
}
