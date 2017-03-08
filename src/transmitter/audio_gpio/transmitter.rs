use std::process::{Command, Stdio};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use raspi::{Gpio, Pin, Direction, Model};

use pocsag::Generator;
use config::Config;
use transmitter::Transmitter;

const BAUD_RATE: usize = 1200;
const SAMPLE_RATE: usize = 48000;
const SAMPLES_PER_BIT: usize = SAMPLE_RATE/BAUD_RATE;

pub struct AudioGpioTransmitter {
    device: String,
    ptt_pin: Pin,
    inverted: bool,
    level: u8,
    tx_delay: usize
}

impl AudioGpioTransmitter {
    pub fn new(config: &Config) -> AudioGpioTransmitter {
        info!("Initializing audio GPIO transmitter...");
        info!("Detected {}", Model::get());

        let gpio = Gpio::new().expect("Failed to map GPIO");

        let device = match &*config.audio_gpio.device {
            "" => String::from("default"),
            other => other.to_owned()
        };

        let mut transmitter = AudioGpioTransmitter {
            device: device,
            ptt_pin: gpio.pin(config.audio_gpio.ptt_pin, Direction::Output),
            inverted: config.audio_gpio.inverted,
            level: config.audio_gpio.level,
            tx_delay: config.audio_gpio.tx_delay
        };

        if transmitter.level > 127 {
            transmitter.level = 127;
        }

       transmitter.ptt_pin.set_low();

       transmitter
    }
}

impl Transmitter for AudioGpioTransmitter {
    fn send(&mut self, gen: Generator) {
        self.ptt_pin.set_high();

        sleep(Duration::from_millis(self.tx_delay as u64));

        let mut buffer: Vec<u8> = Vec::with_capacity(SAMPLE_RATE);
        let low_level = 127 - self.level;
        let high_level = 128 + self.level;

        for word in gen {
            for i in 0..32 {
                let bit = (word & (1 << (31 - i))) != 0;
                if (!self.inverted && bit) || (self.inverted && !bit) {
                    buffer.extend_from_slice(&[low_level; SAMPLES_PER_BIT]);
                }
                else {
                    buffer.extend_from_slice(&[high_level; SAMPLES_PER_BIT]);
                }
            }
        }

        let mut child = Command::new("aplay")
            .stdin(Stdio::piped())
            .args(&["-t", "raw", "-N", "-f", "U8", "-c", "1"])
            .args(&["-r", &*format!("{}", SAMPLE_RATE)])
            .args(&["-D", &*self.device])
            .spawn()
            .expect("Failed to start aplay");

        child.stdin.as_mut()
            .expect("Failed to get aplay stdin")
            .write_all(buffer.as_slice())
            .expect("Failed to write to aplay stdin");

        child.wait().expect("Failed to wait for aplay");

        self.ptt_pin.set_low();
    }
}
