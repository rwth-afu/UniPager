use pocsag::Generator;
use config::Config;
use transmitter::Transmitter;

pub struct BasebandTransmitter {

}

impl BasebandTransmitter {
    pub fn new(_: &Config) -> BasebandTransmitter {
        BasebandTransmitter { }
    }
}

impl Transmitter for BasebandTransmitter {
    fn send(&mut self, _: Generator) {
        info!("Sending data...");
        info!("Data sent.");
    }
}
