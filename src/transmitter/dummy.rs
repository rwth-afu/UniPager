use pocsag::Generator;
use config::Config;
use transmitter::Transmitter;

pub struct DummyTransmitter;

impl DummyTransmitter {
    pub fn new(_: &Config) -> DummyTransmitter {
        warn!("Using dummy transmitter");
        DummyTransmitter { }
    }
}

impl Transmitter for DummyTransmitter {
    fn send(&mut self, gen: Generator) {
        for word in gen {
            info!("{:032b}", word);
        }
    }
}
