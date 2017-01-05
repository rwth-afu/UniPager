#![feature(alloc_system)]
extern crate alloc_system;
extern crate serial;
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
    println!("Copyright (c) 2017 RWTH Amateurfunkgruppe\n");
    println!("This program comes with ABSOLUTELY NO WARRANTY.");
    println!("This is free software, and you are welcome to redistribute");
    println!("and modify it under the conditions of the GNU GPL v3 or later.");
    println!("<https://www.gnu.org/licenses/gpl-3.0.txt>\n");
}

fn main() {
    print_version();

    let (responder, requests) = frontend::run();

    logging::init(responder.clone());

    let config = config::Config::load();

    let scheduler = pocsag::Scheduler::new(&config);

    let server = server::Server::new(&config);

    let scheduler1 = scheduler.clone();
    thread::spawn(move || server.run(scheduler1));

    let config1 = config.clone();
    let scheduler2 = scheduler.clone();
    thread::spawn(move || {
        match config1.transmitter {
            config::Transmitter::Dummy =>
                scheduler2.run(transmitter::DummyTransmitter::new(&config1)),
            config::Transmitter::Audio =>
                scheduler2.run(transmitter::AudioTransmitter::new(&config1)),
            config::Transmitter::Raspager =>
                scheduler2.run(transmitter::RaspagerTransmitter::new(&config1)),
            config::Transmitter::C9000 =>
                scheduler2.run(transmitter::C9000Transmitter::new(&config1))
        };
    });

    use frontend::{Request, Response};
    for req in requests {
        match req {
            Request::SendMessage { addr, data } => {
                let msg = pocsag::Message {
                    id: 0,
                    mtype: pocsag::MessageType::AlphaNum,
                    speed: pocsag::MessageSpeed::Baud(1200),
                    addr: addr,
                    func: pocsag::MessageFunc::AlphaNum,
                    data: data
                };
                scheduler.enqueue(msg);
            }
            Request::GetConfig => {
                responder.send(Response::Config(config.clone()));
            },
            Request::GetVersion => {
                let version = env!("CARGO_PKG_VERSION").to_string();
                responder.send(Response::Version(version));
            },
            Request::Shutdown =>
                break,
            Request::Restart =>
                break,
            req => {
                warn!("Unimplemented request: {:?}", req);
                responder.send(Response::Error("Unimplemented".to_string()));
            }
        }
    }
}
