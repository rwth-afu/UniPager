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

#[macro_use]
mod status;
mod config;
mod logging;
mod connection;
mod core;
mod transmitter;
mod pocsag;
mod frontend;

use std::thread;
use std::time;
use std::fs::File;
use std::io::Read;

use tokio::runtime::Runtime;
use futures::Future;

use config::Config;
use frontend::{Request, Response};
use pocsag::Scheduler;

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
        .map_err(|e| eprintln!("Failed to load password file."))
        .ok();

//    let (responder, requests) = futures::sync::mpsc::unbounded();

    logging::init();
    //status::subscribe(responder.clone());

    let mut config = Config::load();
    let scheduler = Scheduler::new(&config);
    let scheduler_thread = Scheduler::start(config.clone(), scheduler.clone());

    Runtime::new().unwrap().block_on(
        core::bootstrap(&config)
    ).map(|res| {
        println!("{:?}", res);
    });

    let mut rt = Runtime::new().unwrap();
    rt.spawn(frontend::websocket::server(pass));
    rt.spawn(frontend::http::server());
    rt.spawn(connection::consumer(&config, scheduler.clone()));



    rt.shutdown_on_idle().wait().unwrap();

    info!("Terminating... 73!");
}
