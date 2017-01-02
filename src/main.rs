#![feature(proc_macro)]

extern crate serial;
extern crate env_logger;
extern crate raspi;
extern crate ws;
extern crate tiny_http;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate log;

mod config;
mod logging;
mod server;
mod connection;
mod transmitter;
mod pocsag;
mod frontend;

use std::thread;

fn main() {
    logging::init();

    info!("Starting RustPager");

    let config = config::Config::load();

    let scheduler = pocsag::Scheduler::new();

    let scheduler1 = scheduler.clone();
    thread::spawn(move || {
        //let mut transmitter = transmitter::raspager::Transmitter::new();
        let mut transmitter = transmitter::baseband::Transmitter::new();
        transmitter.run();
        scheduler1.run(transmitter)
    });

    let server = server::Server::new(&config);
    let scheduler2 = scheduler.clone();
    let res = thread::spawn(move || server.run(scheduler2));

    thread::spawn(frontend::run);

    res.join().unwrap();
}
