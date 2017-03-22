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

use pocsag::Generator;

pub trait Transmitter {
    fn send(&mut self, Generator);
}
