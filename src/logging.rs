use log::{self, Level, LevelFilter, Log, Metadata, Record};

use crate::event::{Event, EventHandler};

struct Logger {
    event_handler: EventHandler
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
            let event = Event::Log(record.level() as u8, record.args().to_string());
            self.event_handler.publish(event);
        }
    }

    fn flush(&self) {}
}

pub fn init(event_handler: EventHandler) {
    log::set_boxed_logger(Box::new(Logger { event_handler }))
        .expect("Unable to setup logger");
    log::set_max_level(LevelFilter::Trace);
}
