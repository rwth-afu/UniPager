use std::io::Write;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use crate::config::Config;
use crate::transmitter::Ptt;
use crate::transmitter::Transmitter;

const BAUD_RATE: usize = 1200;
const SAMPLE_RATE: usize = 48000;
const SAMPLES_PER_BIT: usize = SAMPLE_RATE / BAUD_RATE;

pub struct AudioTransmitter {
    device: String,
    ptt: Ptt,
    inverted: bool,
    level: u8,
    tx_delay: usize
}

impl AudioTransmitter {
    pub fn new(config: &Config) -> AudioTransmitter {
        info!("Initializing audio transmitter...");

        let device = match &*config.audio.device {
            "" => String::from("default"),
            other => other.to_owned(),
        };

        let mut transmitter = AudioTransmitter {
            device,
            ptt: Ptt::from_config(&config.ptt),
            inverted: config.audio.inverted,
            level: config.audio.level,
            tx_delay: config.audio.tx_delay
        };

        if transmitter.level > 127 {
            transmitter.level = 127;
        }

        transmitter.ptt.set(false);

        transmitter
    }
}

impl Transmitter for AudioTransmitter {
    fn send(&mut self, gen: &mut dyn Iterator<Item = u32>) {
        self.ptt.set(true);

        sleep(Duration::from_millis(self.tx_delay as u64));

        let mut buffer: Vec<u8> = Vec::with_capacity(SAMPLE_RATE);
        let low_level = 127 - self.level;
        let high_level = 128 + self.level;

        for word in gen {
            for i in 0..32 {
                let bit = (word & (1 << (31 - i))) != 0;
                if (!self.inverted && bit) || (self.inverted && !bit) {
                    buffer.extend_from_slice(&[low_level; SAMPLES_PER_BIT]);
                } else {
                    buffer.extend_from_slice(&[high_level; SAMPLES_PER_BIT]);
                }
            }
        }

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
        }
        else {
            error!("Failed to start aplay");
        }

        self.ptt.set(false);
    }
}
