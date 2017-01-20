pub mod dummy;
pub mod audio;
pub mod c9000;
pub mod stm32pager;
pub mod raspager;

pub use self::dummy::DummyTransmitter;
pub use self::audio::AudioTransmitter;
pub use self::c9000::C9000Transmitter;
pub use self::raspager::RaspagerTransmitter;
pub use self::stm32pager::STM32Transmitter;

use pocsag::Generator;

pub trait Transmitter {
    fn send(&mut self, Generator);
}
