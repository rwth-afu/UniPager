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

use std::thread::{self, JoinHandle};
use std::time;

use config::Config;
use pocsag::Scheduler;
use frontend::{Request, Response};

fn print_version() {
    println!("RustPager {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (c) 2017 RWTH Amateurfunkgruppe\n");
    println!("This program comes with ABSOLUTELY NO WARRANTY.");
    println!("This is free software, and you are welcome to redistribute");
    println!("and modify it under the conditions of the GNU GPL v3 or later.");
    println!("<https://www.gnu.org/licenses/gpl-3.0.txt>\n");
}

pub fn run_scheduler(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
    use transmitter::*;
    thread::spawn(move || {
        match config.transmitter {
            config::Transmitter::Dummy =>
                scheduler.run(DummyTransmitter::new(&config)),
            config::Transmitter::Audio =>
                scheduler.run(AudioTransmitter::new(&config)),
            config::Transmitter::Raspager =>
                scheduler.run(RaspagerTransmitter::new(&config)),
            config::Transmitter::C9000 =>
                scheduler.run(C9000Transmitter::new(&config))
        };
    })
}

pub fn run_server(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
    let server = server::Server::new(&config);
    thread::spawn(move || server.run(scheduler))
}

fn main() {
    print_version();

    let (responder, requests) = frontend::run();

    logging::init(responder.clone());

    let mut config = Config::load();
    let scheduler = Scheduler::new(&config);

    run_server(config.clone(), scheduler.clone());

    let mut restart = true;
    while restart {
        let scheduler_thread = run_scheduler(config.clone(), scheduler.clone());
        loop {
            match requests.recv().unwrap() {
                Request::SetConfig(new_config) => {
                    config = new_config;
                },
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
                },
                Request::GetConfig => {
                    responder.send(Response::Config(config.clone()));
                },
                Request::GetVersion => {
                    let version = env!("CARGO_PKG_VERSION").to_string();
                    responder.send(Response::Version(version));
                },
                Request::Shutdown => {
                    restart = false;
                    scheduler.stop();
                    info!("Initiating shutdown.");
                    break;
                },
                Request::Restart => {
                    restart = true;
                    scheduler.stop();
                    info!("Initiating restart.");
                    break;
                }
            }
        }

        info!("Waiting for the scheduler to terminate...");
        scheduler_thread.join().unwrap();
        info!("Scheduler stopped.");
    }

    info!("Terminating... 73!");
    thread::sleep(time::Duration::from_millis(1000));
}
