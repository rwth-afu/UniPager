use std::process::{Command, Stdio};
use std::io::Write;
use raspi::{Gpio, Pin, Direction, Model};

use pocsag::Generator;
use config::Config;
use transmitter::Transmitter;

const BAUD_RATE: usize = 1200;
const SAMPLE_RATE: usize = 48000;
const SAMPLES_PER_BIT: usize = SAMPLE_RATE/BAUD_RATE;

pub struct AudioTransmitter {
    ptt_pin: Pin
}

impl AudioTransmitter {
    pub fn new(_: &Config) -> AudioTransmitter {
        info!("Initializing audio transmitter...");
        info!("Detected {}", Model::get());

        let gpio = Gpio::new().expect("Failed to map GPIO");

        let transmitter = AudioTransmitter {
            ptt_pin: gpio.pin(0, Direction::Output)
        };

        transmitter.ptt_pin.set_low();

        transmitter
    }
}

impl Transmitter for AudioTransmitter {
    fn send(&mut self, gen: Generator) {
        info!("Sending data...");

        let mut buffer: Vec<u8> = Vec::with_capacity(SAMPLE_RATE);

        for word in gen {
            for i in 0..32 {
                let bit = word & (1 << (31 - i));
                if bit == 0 {
                    buffer.extend_from_slice(&[0; SAMPLES_PER_BIT]);
                }
                else {
                    buffer.extend_from_slice(&[255; SAMPLES_PER_BIT]);
                }
            }
        }

        self.ptt_pin.set_high();

        let mut child = Command::new("aplay")
            .stdin(Stdio::piped())
            .args(&["-t", "raw", "-N", "-f", "U8", "-c", "1"])
            .args(&["-r", &*format!("{}", SAMPLE_RATE)])
            .spawn()
            .expect("Failed to start aplay");

        child.stdin.as_mut()
            .expect("Failed to get aplay stdin")
            .write_all(buffer.as_slice())
            .expect("Failed to write to aplay stdin");

        child.wait().expect("Failed to wait for aplay");

        self.ptt_pin.set_low();

        info!("Data sent.");
    }
}
