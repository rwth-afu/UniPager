use pocsag::Generator;

pub struct Transmitter {

}

impl Transmitter  {
    pub fn new() -> Transmitter {
        Transmitter { }
    }

    pub fn run(&mut self) {

    }
}

impl ::transmitter::Transmitter for Transmitter {
    fn send(&mut self, gen: Generator) {
        info!("Sending data...");

        for word in gen {

        }
        info!("Data sent.");
    }
}
