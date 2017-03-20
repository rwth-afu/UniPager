#![feature(alloc_system, mpsc_select)]
extern crate alloc_system;
extern crate serial;
extern crate raspi;
extern crate ws;
extern crate tiny_http;
extern crate net2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod status;
mod config;
mod logging;
mod connection;
mod transmitter;
mod pocsag;
mod frontend;

use std::thread;
use std::time;

use config::Config;
use pocsag::Scheduler;
use frontend::{Request, Response};
use connection::Connection;

fn print_version() {
    println!("UniPager {}", env!("CARGO_PKG_VERSION"));
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
    status::subscribe(responder.clone());

    let mut config = Config::load();
    let scheduler = Scheduler::new(&config);

    let mut restart = true;
    while restart {
        let (stop_conn, conn_thread) = Connection::start(config.clone(), scheduler.clone());
        let scheduler_thread = Scheduler::start(config.clone(), scheduler.clone());
        loop {
            match requests.recv().unwrap() {
                Request::SetConfig(new_config) => {
                    config = new_config;
                    config.save();
                    info!("Config updated. Initiating restart.");

                    restart = true;
                    scheduler.stop();
                    stop_conn.send(()).ok();
                    break;
                },
                Request::DefaultConfig => {
                    config = Config::default();
                    config.save();
                    info!("Config set to default. Initiating restart.");

                    restart = true;
                    scheduler.stop();
                    stop_conn.send(()).ok();
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
                Request::GetStatus => {
                    responder.send(Response::Status(status::get()));
                },
                Request::Shutdown => {
                    info!("Initiating shutdown.");
                    restart = false;
                    scheduler.stop();
                    stop_conn.send(()).ok();
                    break;
                },
                Request::Restart => {
                    info!("Initiating restart.");
                    restart = true;
                    scheduler.stop();
                    stop_conn.send(()).ok();
                    break;
                }
            }
        }

        info!("Waiting for the connection to terminate...");
        conn_thread.join().ok();

        info!("Waiting for the scheduler to terminate...");
        scheduler_thread.join().ok();
        info!("Scheduler stopped.");
    }

    info!("Terminating... 73!");
    thread::sleep(time::Duration::from_millis(1000));
}
