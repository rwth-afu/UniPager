use env_logger;
use log;

pub fn init() {
    let format = |record: &log::LogRecord| {
        let color = match record.level() {
            log::LogLevel::Error => "\x1B[31m",
            log::LogLevel::Warn => "\x1B[33m",
            log::LogLevel::Info => "\x1B[32m",
            _ => ""
        };
        format!("\r{}{}\x1B[39m - {}", color, record.level(), record.args())
    };

    let mut builder = env_logger::LogBuilder::new();
    builder.format(format).filter(Some("rustpager"), log::LogLevelFilter::Info);

    if let Ok(ref value) = ::std::env::var("RUST_LOG") {
       builder.parse(value);
    }

    builder.init().unwrap();
}
