const MMDVM_FRAME_START: u8 = 0xE0;
const MMDVM_GET_VERSION: u8 = 0x00;

use serial::{self, SerialPort};

use config::Config;
use transmitter::Transmitter;
use std::io::{Write};

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

        let bytes = [
            MMDVM_FRAME_START as u8,
            3 as u8,
            MMDVM_GET_VERSION as u8,
        ];
        if serial.write_all(&bytes).is_err() {
            error!("Unable to send end of transmission byte");
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
