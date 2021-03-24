use std::io::Write;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use crate::config::Config;
use crate::transmitter::Ptt;
use crate::transmitter::Transmitter;

const SAMPLE_RATE: usize = 48000;

pub struct AudioTransmitter {
    device: String,
    ptt: Ptt,
    inverted: bool,
    level: u8,
    tx_delay: usize,
    samples_per_bit: usize,
}

impl AudioTransmitter {
    pub fn new(config: &Config) -> AudioTransmitter {
        info!("Initializing audio transmitter with baudrate '{}'...", config.audio.baudrate);

        let device = match &*config.audio.device {
            "" => String::from("default"),
            other => other.to_owned(),
        };

        let mut transmitter = AudioTransmitter {
            device,
            ptt: Ptt::from_config(&config.ptt),
            inverted: config.audio.inverted,
            level: config.audio.level,
            tx_delay: config.audio.tx_delay,
            samples_per_bit: SAMPLE_RATE / config.audio.baudrate,
        };

        if transmitter.level > 127 {
            transmitter.level = 127;
        }

        transmitter.ptt.set(false);

        transmitter
    }
}

impl Transmitter for AudioTransmitter {
    fn send(&mut self, gen: &mut dyn Iterator<Item=u32>) {
        trace!("Activating PTT to start transmission.");
        self.ptt.set(true);

        trace!("Waiting for {}ms before audio transmission starts.", self.tx_delay);
        sleep(Duration::from_millis(self.tx_delay as u64));

        let mut buffer: Vec<u8> = Vec::with_capacity(SAMPLE_RATE);
        let low_level = 127 - self.level;
        let high_level = 128 + self.level;
        trace!(
            "Sending with low_level='{}' and high_level='{}' based on configured level='{}'.",
            low_level,
            high_level,
            self.level);

        let low_bit_sample = create_bit_sample(self.samples_per_bit, low_level);
        let high_bit_sample = create_bit_sample(self.samples_per_bit, high_level);
        for word in gen {
            for i in 0..32 {
                let bit = (word & (1 << (31 - i))) != 0;
                if (!self.inverted && bit) || (self.inverted && !bit) {
                    buffer.extend(&low_bit_sample);
                } else {
                    buffer.extend(&high_bit_sample);
                }
            }
        }

        trace!("Spawning `aplay` child process to start audio transmission.");
        let child = Command::new("aplay")
            .stdin(Stdio::piped())
            .args(&["-t", "raw", "-N", "-f", "U8", "-c", "1"])
            .args(&["-r", &*format!("{}", SAMPLE_RATE)])
            .args(&["-D", &*self.device])
            .spawn();

        if let Ok(mut child) = child {
            let result = child
                .stdin
                .as_mut()
                .and_then(|stdin| {
                    stdin.write_all(buffer.as_slice()).ok()
                });

            if result.is_none() {
                error!("Failed to write to aplay stdin")
            }

            child.wait().ok();
        } else {
            error!("Failed to start aplay");
        }

        trace!("Deactivating PTT to end transmission.");
        self.ptt.set(false);
    }
}

fn create_bit_sample(sample_size: usize, constant_value: u8) -> Vec<u8> {
    let mut bit_sample: Vec<u8> = Vec::with_capacity(sample_size);
    for _s in 0..sample_size {
        bit_sample.push(constant_value);
    }

    return bit_sample;
}
