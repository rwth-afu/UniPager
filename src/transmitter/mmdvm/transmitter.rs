const MMDVM_FRAME_START: u8 = 0xE0;
const MMDVM_GET_VERSION: u8 = 0x00;
const MMDVM_SET_CONFIG:  u8 = 0x02;
const MODE_IDLE:         u8 = 0x00;

const BUFFER_LENGTH:     usize = 2000;

use serial::{self, SerialPort};

use config::Config;
use transmitter::Transmitter;
use std::io::{Read,Write};

pub struct MMDVMTransmitter {
    serial: Box<serial::SerialPort>
}

impl MMDVMTransmitter {
    pub fn new(config: &Config) -> MMDVMTransmitter {
        info!("Initializing MMDVM transmitter...");

        let mut serial = serial::open(&config.mmdvm.port).expect(
            "Unable to open serial port"
        );

        serial
            .configure(&serial::PortSettings {
                baud_rate: serial::BaudRate::Baud115200,
                char_size: serial::CharSize::Bits8,
                parity: serial::Parity::ParityNone,
                stop_bits: serial::StopBits::Stop1,
                flow_control: serial::FlowControl::FlowNone
            })
            .expect("Unable to configure serial port");

        let inverted = config.mmdvm.inverted;
        let mut level = config.mmdvm.level;
        if level > 100.0 {
            level = 100.0;
        }
        level = level * 2.55 + 0.5;

        let mut buffer3 = 0;
        if inverted {
            buffer3 |= 0x01;
        }

        let bytes = [
            MMDVM_FRAME_START as u8,
            3 as u8,
            MMDVM_GET_VERSION as u8,
        ];
        let mut buffer = [0; BUFFER_LENGTH];
        let mut length = 0;
        if serial.write_all(&bytes).is_err() {
            error!("Unable to intialize MMDVM!");
        }

        let mut offset = 0;
        if !serial.read(&mut buffer[..]).is_err() {
            // println!("Curr {:?}", buffer[0]);
            if offset == 0 {
                let ret = buffer[offset];
                if ret < 0 {
                    error!("Error when reading from the modem");
                }
                if ret == 0 {
                    error!("Timeout");
                }
                if ret != MMDVM_FRAME_START {
                    error!("Timeout");
                } else {
                    println!("MMDVM Frame Start found");
                }
                offset = 1;
            }
            if offset == 1 {
                // Get the length of the frame
                length = buffer[offset];
                println!("Frame length: {}", length);
                offset = 2;
            }
            if offset == 2 {
                // Get the frame type
                let ret = buffer[offset];
                if ret < 0 {
                    offset = 0;
                    error!("Error when reading from the modem");
                }
                if ret == 0 {
                    error!("Timeout");
                }
            }

        } else {
            error!("Error when reading from the modem");
        }

        let bytes = [
            MMDVM_FRAME_START as u8,
            // Length is 21 bytes
            21 as u8,
            MMDVM_SET_CONFIG as u8,
            // Invert, deviation and duplex settings
            buffer3,
            // Enable POCSAG and disable all other modes
            0x20 as u8,
            // TXdelay in 10ms units
            10 as u8,
            // Idle mode
            MODE_IDLE as u8,
            // RXLevel (not needed)
            0 as u8,
            // CW ID TX level (not needed)
            0 as u8,
            // DMR color code (not needed)
            0 as u8,
            // DMR delay (not needed)
            0 as u8,
            // Was OscOffset (not needed)
            128 as u8,
            // DStar TX Level (not needed)
            0 as u8,
            // DMR TX Level (not needed)
            0 as u8,
            // YSF TX Level (not needed)
            0 as u8,
            // P25 TX Level (not needed)
            0 as u8,
            // TX DC offset (not needed)
            0 as u8,
            // RX DC offset (not needed)
            0 as u8,
            // NXDN TX Level (not needed)
            0 as u8,
            // YSF TX hang time (not needed)
            0 as u8,
            // POCSAG TX level
            level as u8
        ];
        if serial.write_all(&bytes).is_err() {
            error!("Unable to set MMDVM config!");
        }

        MMDVMTransmitter { serial: Box::new(serial) }
    }
}

impl Transmitter for MMDVMTransmitter {
    fn send(&mut self, gen: &mut Iterator<Item = u32>) {
        for word in gen {
            let bytes = [
                ((word & 0xff000000) >> 24) as u8,
                ((word & 0x00ff0000) >> 16) as u8,
                ((word & 0x0000ff00) >> 8) as u8,
                (word & 0x000000ff) as u8,
            ];

            if (*self.serial).write_all(&bytes).is_err() {
                error!("Unable to write data to the serial port");
                return;
            }
        }

        // Send End of Transmission packet
        let eot = [0x17 as u8];
        if (*self.serial).write_all(&eot).is_err() {
            error!("Unable to send end of transmission byte");
            return;
        }

        if (*self.serial).flush().is_err() {
            error!("Unable to flush serial port");
        }
    }
}
