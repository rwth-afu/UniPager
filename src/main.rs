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

fn print_version() {
    println!("RustPager {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (c) 2016 RWTH Amateurfunk Gruppe\n");
    println!("This program comes with ABSOLUTELY NO WARRANTY.");
    println!("This is free software, and you are welcome to redistribute");
    println!("and modify it under the conditions of the GNU GPL v3 or later.");
    println!("<https://www.gnu.org/licenses/gpl-3.0.txt>\n");
}

fn main() {
    print_version();
    logging::init();

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
