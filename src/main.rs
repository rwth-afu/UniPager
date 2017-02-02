#![feature(alloc_system, duration_checked_ops)]
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
                scheduler.run(C9000Transmitter::new(&config)),
            config::Transmitter::STM32Pager =>
                scheduler.run(STM32Transmitter::new(&config))
        };
    })
}

pub fn run_connection(config: Config, scheduler: Scheduler) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            info!("Trying to connect to Master...");
            let connection = connection::Connection::new(&config, scheduler.clone());
            if let Ok(mut connection) = connection {
                info!("Connection established.");
                connection.run();
                info!("Connection lost.");
            }
            thread::sleep(time::Duration::from_millis(5000));
        }
    })
}

fn main() {
    print_version();

    let (responder, requests) = frontend::run();

    logging::init(responder.clone());

    let mut config = Config::load();
    let scheduler = Scheduler::new(&config);

    run_connection(config.clone(), scheduler.clone());

    let mut restart = true;
    while restart {
        let scheduler_thread = run_scheduler(config.clone(), scheduler.clone());
        loop {
            match requests.recv().unwrap() {
                Request::SetConfig(new_config) => {
                    config = new_config;
                    config.save();
                    info!("Config updated. Initiating restart.");
                    restart = true;
                    scheduler.stop();
                    break;
                },
                Request::DefaultConfig => {
                    config = Config::default();
                    config.save();
                    info!("Config set to default. Initiating restart.");
                    restart = true;
                    scheduler.stop();
                    break;
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
                    scheduler.message(msg);
                },
                Request::GetConfig => {
                    responder.send(Response::Config(config.clone()));
                },
                Request::GetVersion => {
                    let version = env!("CARGO_PKG_VERSION").to_string();
                    responder.send(Response::Version(version));
                },
                Request::Shutdown => {
                    info!("Initiating shutdown.");
                    restart = false;
                    scheduler.stop();
                    break;
                },
                Request::Restart => {
                    info!("Initiating restart.");
                    restart = true;
                    scheduler.stop();
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
