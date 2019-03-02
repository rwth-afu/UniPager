const MMDVM_FRAME_START: u8 = 0xE0;
const MMDVM_GET_VERSION: u8 = 0x00;
const MMDVM_SET_CONFIG:  u8 = 0x02;
const MMDVM_SET_MODE:    u8 = 0x03;
const MMDVM_SET_FREQ:    u8 = 0x04;
const MMDVM_POCSAG_DATA: u8 = 0x50;
const MMDVM_MODE_IDLE:   u8 = 0x00;
const MMDVM_ACK:         u8 = 0x70;
const MMDVM_NACK:        u8 = 0x7F;
const BUFFER_LENGTH:     usize = 2000;

use config::Config;
use transmitter::Transmitter;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;
use serialport::prelude::*;

pub struct MMDVMTransmitter {
    serial: Box<dyn SerialPort>
}

impl MMDVMTransmitter {
    pub fn new(config: &Config) -> MMDVMTransmitter {
        info!("Initializing MMDVM transmitter...");

        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(500);
        settings.baud_rate = 115200;

        let mut serial = serialport::open_with_settings(&config.mmdvm.port, &settings)
            .expect("Unable to open serial port");

        let mut tx = MMDVMTransmitter { serial: serial };
        tx.init(config);
        tx
    }

    pub fn init(&mut self, config: &Config) {
        let inverted = config.mmdvm.inverted;
        let mut level = config.mmdvm.level;

        level = if level > 100.0 { 100.0 } else { level };
        level *= 2.55 + 0.5;

        self.send_cmd(MMDVM_GET_VERSION, &[]);

        let mut buffer = [0; BUFFER_LENGTH];
        if !self.serial.read(&mut buffer[..]).is_err() {
            match &buffer[0..4] {
              [MMDVM_FRAME_START, length, MMDVM_GET_VERSION, version] => {
                info!("Received version {:?} length {:?}", version, length);
              },
              _ => warn!("Unknown frame received")
            }
        } else {
            error!("Error when reading from the modem");
        }

        self.send_cmd(MMDVM_SET_CONFIG, &[
            // Invert, deviation and duplex settings
            (inverted as u8) << 4 | 0x80,
            // Enable POCSAG and disable all other modes
            0x20,
            // TXdelay in 10ms units
            10,
            // Idle mode
            MMDVM_MODE_IDLE,
            // RXLevel (not needed)
            0,
            // CW ID TX level (not needed)
            0,
            // DMR color code (not needed)
            0,
            // DMR delay (not needed)
            0,
            // Was OscOffset (not needed)
            128,
            // DStar TX Level (not needed)
            0,
            // DMR TX Level (not needed)
            0,
            // YSF TX Level (not needed)
            0,
            // P25 TX Level (not needed)
            0,
            // TX DC offset (not needed)
            0,
            // RX DC offset (not needed)
            0,
            // NXDN TX Level (not needed)
            0,
            // YSF TX hang time (not needed)
            0,
            // POCSAG TX level
            level as u8
        ]);

        self.read_result();

        self.send_cmd(MMDVM_SET_FREQ, &[
            0x0,
            // freq_rx
            0x2C, 0xAD, 0x39, 0x1A,
            // freq_tx
            0x2C, 0xAD, 0x39, 0x1A,
            // rf_power,
            0xFF,
            // pocsag_freq_tx
            0x2C, 0xAD, 0x39, 0x1A,
        ]);

        self.read_result();
    }

    pub fn send_cmd(&mut self, cmd: u8, data: &[u8]) {
        let header = [
            MMDVM_FRAME_START,
            (data.len() + 3) as u8,
            cmd
        ];

        if self.serial.write_all(&header).is_err() {
            error!("Failed to write to MMDVM.");
            return;
        }

        if self.serial.write_all(&data).is_err() {
            error!("Failed to write to MMDVM.");
            return;
        }

        if self.serial.flush().is_err() {
            error!("Unable to flush serial port");
        }
    }

    pub fn read_result(&mut self) -> Result<u8, Option<u8>> {
        let mut buffer = [0; BUFFER_LENGTH];

        for i in 0..50 {
            if let Ok(len) = self.serial.bytes_to_read() {
                if len > 4 {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        if let Ok(len) = self.serial.read(&mut buffer[..]) {
            match &buffer[0..4] {
                [MMDVM_FRAME_START, _, MMDVM_ACK, ftype] => {
                    info!("Received ACK ({:?})", ftype);
                    Ok(*ftype)
                }
                [MMDVM_FRAME_START, _, MMDVM_NACK, ftype] => {
                    warn!("Received NACK ({:?}): {:?}", ftype, buffer.get(4));
                    if let Some(err) = buffer.get(4) {
                        Err(Some(*err))
                    }
                    else {
                        Err(None)
                    }
                }
                _ => {
                    warn!("Unknown frame received");
                    Err(None)
                }
            }
        }
        else {
            error!("Error when reading from the modem");
            Err(None)
        }
    }
}

impl Transmitter for MMDVMTransmitter {
    fn send(&mut self, gen: &mut Iterator<Item = u32>) {
        let mut buffer: Vec<u8> = Vec::with_capacity(252);

        loop {
            buffer.clear();

            for word in gen.take(17) {
                let bytes = [
                    ((word & 0xff000000) >> 24) as u8,
                    ((word & 0x00ff0000) >> 16) as u8,
                    ((word & 0x0000ff00) >> 8) as u8,
                    (word & 0x000000ff) as u8,
                ];

                buffer.extend_from_slice(&bytes);
            }

            if !buffer.is_empty() {
                let start = [
                    MMDVM_FRAME_START,
                    (buffer.len() + 3) as u8,
                    MMDVM_POCSAG_DATA as u8,
                ];

                let mut result = Err(Some(5));
                while result == Err(Some(5)) {
                    if self.serial.write_all(&start).is_err() {
                        error!("Unable to intialize MMDVM!");
                    }

                    if self.serial.write_all(&buffer[..]).is_err() {
                        error!("Unable to intialize MMDVM!");
                    }

                    if self.serial.flush().is_err() {
                        error!("Unable to flush serial port");
                    }

                    result = self.read_result();
                }
            }
            else {
                break;
            }
        }

        self.send_cmd(MMDVM_SET_MODE, &[MMDVM_MODE_IDLE]);

        self.read_result();
    }
}
