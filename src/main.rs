extern crate sysfs_gpio;
extern crate serial;
extern crate toml;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rustyline;

mod baseband;
mod c9000;
mod raspager;
mod config;
mod logging;
mod server;
mod connection;
mod message;
mod scheduler;
mod timeslots;
mod prompt;
mod transmitter;
mod generator;

use std::thread;

fn main() {
    logging::init();

    //let mut generator = raspi::Generator::new();
    //generator.send("Hello World!");

    let mut transmitter = raspager::Transmitter::new();
    transmitter.run();
    // let mut transmitter = baseband::Transmitter::new();

    info!("Starting RustPager");

    let config = config::Config::load();

    let scheduler = scheduler::Scheduler::new();

    let scheduler1 = scheduler.clone();
    thread::spawn(move || scheduler1.run(transmitter));

    let server = server::Server::new(&config);
    let scheduler2 = scheduler.clone();
    let res = thread::spawn(move || server.run(scheduler2));

    prompt::run(scheduler);

    res.join().unwrap();
}
