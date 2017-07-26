pub mod ptt;
pub mod dummy;
pub mod audio;
pub mod c9000;
pub mod rfm69;
pub mod raspager;

pub use self::ptt::Ptt;
pub use self::dummy::DummyTransmitter;
pub use self::audio::AudioTransmitter;
pub use self::c9000::C9000Transmitter;
pub use self::raspager::RaspagerTransmitter;
pub use self::rfm69::RFM69Transmitter;

use config::{self, Config};

pub trait Transmitter {
    fn send(&mut self, &mut Iterator<Item=u32>);
}


pub fn from_config(config: &Config) -> Box<Transmitter> {
    match config.transmitter {
        config::Transmitter::Dummy =>
            Box::new(DummyTransmitter::new(config)) as Box<Transmitter>,
        config::Transmitter::Audio =>
            Box::new(AudioTransmitter::new(config)) as Box<Transmitter>,
        config::Transmitter::Raspager =>
            Box::new(RaspagerTransmitter::new(config)) as Box<Transmitter>,
        config::Transmitter::C9000 =>
            Box::new(C9000Transmitter::new(config)) as Box<Transmitter>,
        config::Transmitter::RFM69 =>
            Box::new(RFM69Transmitter::new(config)) as Box<Transmitter>
    }
}
