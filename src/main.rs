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

    thread::spawn(frontend::run);

    let server = server::Server::new(&config);
    let scheduler1 = scheduler.clone();
    let res = thread::spawn(move || server.run(scheduler1));

    thread::spawn(move || {
        match config.transmitter {
            config::Transmitter::Dummy =>
                scheduler.run(transmitter::DummyTransmitter::new(&config)),
            config::Transmitter::Baseband =>
                scheduler.run(transmitter::BasebandTransmitter::new(&config)),
            config::Transmitter::Raspager =>
                scheduler.run(transmitter::RaspagerTransmitter::new(&config)),
            config::Transmitter::C9000 =>
                scheduler.run(transmitter::C9000Transmitter::new(&config))
        };
    });

    res.join().unwrap();
}
