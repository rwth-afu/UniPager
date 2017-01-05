use pocsag::Generator;
use config::Config;
use transmitter::Transmitter;

pub struct AudioTransmitter {

}

impl AudioTransmitter {
    pub fn new(_: &Config) -> AudioTransmitter {
        AudioTransmitter { }
    }
}

impl Transmitter for AudioTransmitter {
    fn send(&mut self, _: Generator) {
        info!("Sending data...");
        info!("Data sent.");
    }
}
