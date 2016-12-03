use generator::Generator;

pub trait Transmitter {
    fn send(&mut self, Generator);
}
