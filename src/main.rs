#![feature(alloc_system, mpsc_select, tcpstream_connect_timeout)]
extern crate alloc_system;
extern crate serial;
extern crate raspi;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate hyper;
extern crate tokio;
extern crate futures;
extern crate lapin_futures as lapin;
extern crate tungstenite;
extern crate tokio_tungstenite;
extern crate tokio_retry;
extern crate chrono;

#[macro_use]
mod telemetry;
mod config;
mod logging;
mod connection;
mod core;
mod transmitter;
mod pocsag;
mod frontend;
mod message;
mod scheduler;
mod timeslots;
mod queue;
mod event;

use std::fs::File;
use std::io::Read;

use futures::Future;
use tokio::runtime::Runtime;

fn print_version() {
    println!("UniPager {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (c) 2017-2018 RWTH Amateurfunkgruppe\n");
    println!("This program comes with ABSOLUTELY NO WARRANTY.");
    println!("This is free software, and you are welcome to redistribute");
    println!("and modify it under the conditions of the GNU GPL v3 or later.");
    println!("<https://www.gnu.org/licenses/gpl-3.0.txt>\n");
}

fn main() {
    print_version();

    let pass = File::open("password")
        .and_then(|mut f| {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            Ok(s)
        })
        .map(|s| s.trim().to_owned())
        .map_err(|_| eprintln!("Failed to load password file."))
        .ok();

    let config = config::get();

    let mut rt = Runtime::new().unwrap();
    let event_handler = event::start(&mut rt);

    logging::init(event_handler.clone());
    scheduler::start(config.clone(), event_handler.clone());
    telemetry::start(&mut rt, event_handler.clone());
    timeslots::start(&mut rt, event_handler.clone());
    frontend::websocket::start(&mut rt, pass, event_handler.clone());
    frontend::http::start(&mut rt, event_handler.clone());
    core::start(&mut rt, &config, event_handler.clone());
    connection::start(&mut rt, &config, event_handler.clone());

    rt.shutdown_on_idle().wait().unwrap();

    info!("Terminating... 73!");
}
