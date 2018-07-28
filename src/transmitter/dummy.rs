use std::thread::sleep;
use std::time::Duration;

use config::Config;
use transmitter::Transmitter;

pub struct DummyTransmitter;

impl DummyTransmitter {
    pub fn new(_: &Config) -> DummyTransmitter {
        warn!("Using dummy transmitter");
        DummyTransmitter {}
    }
}

impl Transmitter for DummyTransmitter {
    fn send(&mut self, gen: &mut Iterator<Item = u32>) {
        let mut count = 0;
        for word in gen {
            info!("{:032b}", word);
            count += 1;
        }

        sleep(Duration::from_millis(count * 3));
    }
}
