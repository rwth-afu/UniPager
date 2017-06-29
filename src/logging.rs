use std::sync::Mutex;
use log::{self, Log, LogRecord, LogLevel, LogLevelFilter, LogMetadata};
use frontend::{Responder, Response};

struct Logger {
    responder: Mutex<Responder>
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug &&
        metadata.target().starts_with("unipager")
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                LogLevel::Error => "\x1B[31m",
                LogLevel::Warn => "\x1B[33m",
                LogLevel::Info => "\x1B[32m",
                _ => ""
            };
            println!("{}{}\x1B[39m - {}", color, record.level(), record.args());
            let res = Response::Log(record.level() as u8, record.args().to_string());
            self.responder.lock().unwrap().send(res);
        }
    }
}

pub fn init(responder: Responder) {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Debug);
        Box::new(Logger { responder: Mutex::new(responder) })
    }).expect("Unable to setup logger");
}
