pub mod baseband;
pub mod c9000;
pub mod raspager;

use pocsag::Generator;

pub trait Transmitter {
    fn send(&mut self, Generator);
}
