use env_logger;
use log;

pub fn init() {
    let format = |record: &log::LogRecord| {
        format!("{} - {}", record.level(), record.args())
    };

    let mut builder = env_logger::LogBuilder::new();
    builder.format(format).filter(None, log::LogLevelFilter::Info);

    if let Ok(ref value) = ::std::env::var("RUST_LOG") {
       builder.parse(value);
    }

    builder.init().unwrap();
}
