//use frontend::{Responder, Response};
use log::{self, Log, Level, LevelFilter, Metadata, Record};
use std::sync::Mutex;

struct Logger {
//    responder: Mutex<Responder>
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info &&
            metadata.target().starts_with("unipager")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                Level::Error => "\x1B[31m",
                Level::Warn => "\x1B[33m",
                Level::Info => "\x1B[32m",
                _ => "",
            };
            println!("{}{}\x1B[39m - {}", color, record.level(), record.args());
//            let res =
 //               Response::Log(record.level() as u8, record.args().to_string());
 //           self.responder.lock().unwrap().send(res);
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_boxed_logger(Box::new(
        Logger { }
    )).expect("Unable to setup logger");
    log::set_max_level(LevelFilter::Info);
}
