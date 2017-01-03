pub mod dummy;
pub mod baseband;
pub mod c9000;
pub mod raspager;

pub use self::dummy::DummyTransmitter;
pub use self::baseband::BasebandTransmitter;
pub use self::c9000::C9000Transmitter;
pub use self::raspager::RaspagerTransmitter;

use pocsag::Generator;

pub trait Transmitter {
    fn send(&mut self, Generator);
}
