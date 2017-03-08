use std::process::{Command, Stdio};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use serial::{self};

use pocsag::Generator;
use config::Config;
use transmitter::Transmitter;

const BAUD_RATE: usize = 1200;
const SAMPLE_RATE: usize = 48000;
const SAMPLES_PER_BIT: usize = SAMPLE_RATE/BAUD_RATE;

pub struct AudioRs232Transmitter {
    device: String,
    ptt_port: String,
    ptt_pin: String,
    ptt_inverted: bool,
    inverted: bool,
    level: u8,
    tx_delay: usize,
    serial: Box<serial::SerialPort>
}

impl AudioRs232Transmitter {
    pub fn new(config: &Config) -> AudioRs232Transmitter {
        info!("Initializing audio RS-232 transmitter...");

        let ptt_port = &config.audio_rs232.ptt_port;
        let ptt_pin = &config.audio_rs232.ptt_pin;
        info!("Initializing serial port {:?} pin {:?}", ptt_port, ptt_pin);
        let serial = serial::open(&config.audio_rs232.ptt_port)
            .expect("Unable to open serial port");

        let device = match &*config.audio_rs232.device {
            "" => String::from("default"),
            other => other.to_owned()
        };

        let mut transmitter = AudioRs232Transmitter {
            device: device,
            ptt_port: config.audio_rs232.ptt_port.to_owned(),
            ptt_pin: config.audio_rs232.ptt_pin.to_owned(),
            ptt_inverted: config.audio_rs232.ptt_inverted.to_owned(),
            inverted: config.audio_rs232.inverted,
            level: config.audio_rs232.level,
            tx_delay: config.audio_rs232.tx_delay,
            serial: Box::new(serial)
        };

        if transmitter.level > 127 {
            transmitter.level = 127;
        }

        if transmitter.ptt_pin == "DTR" {
            if transmitter.ptt_inverted == true {
                transmitter.serial.set_dtr(true)
                   .expect("Error setting PTT pin");
            } else {
                transmitter.serial.set_dtr(false)
                   .expect("Error setting PTT pin");
            }
        } else if transmitter.ptt_pin == "RTS" {
            if transmitter.ptt_inverted == true {
                transmitter.serial.set_rts(true)
                   .expect("Error setting PTT pin");
            } else {
                transmitter.serial.set_rts(false)
                   .expect("Error setting PTT pin");
            }
        }

       transmitter
    }
}

impl Transmitter for AudioRs232Transmitter {
    fn send(&mut self, gen: Generator) {
        if self.ptt_pin == "DTR" {
            if self.ptt_inverted == true {
                self.serial.set_dtr(false)
                   .expect("Error setting PTT pin");
            } else {
                self.serial.set_dtr(true)
                   .expect("Error setting PTT pin");
            }
        } else if self.ptt_pin == "RTS" {
            if self.ptt_inverted == true {
                self.serial.set_rts(false)
                   .expect("Error setting PTT pin");
            } else {
                self.serial.set_rts(true)
                   .expect("Error setting PTT pin");
            }
        }

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

        if self.ptt_pin == "DTR" {
            if self.ptt_inverted == true {
                self.serial.set_dtr(true)
                   .expect("Error setting PTT pin");
            } else {
                self.serial.set_dtr(false)
                   .expect("Error setting PTT pin");
            }
        } else if self.ptt_pin == "RTS" {
            if self.ptt_inverted == true {
                self.serial.set_rts(true)
                   .expect("Error setting PTT pin");
            } else {
                self.serial.set_rts(false)
                   .expect("Error setting PTT pin");
            }
        }

    }
}
