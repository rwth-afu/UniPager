use std::{thread, time};
use serial::{self, SerialPort};

use config::Config;
use pocsag::Generator;
use transmitter::Transmitter;

pub struct STM32Transmitter {
    serial: Box<serial::SerialPort>
}

impl STM32Transmitter  {
    pub fn new(config: &Config) -> STM32Transmitter {
        info!("Initializing STM32Pager transmitter...");

        let mut serial = serial::open(&config.stm32pager.port)
            .expect("Unable to open serial port");

        serial.configure(&serial::PortSettings {
            baud_rate: serial::BaudRate::Baud38400,
            char_size: serial::CharSize::Bits8,
            parity: serial::Parity::ParityNone,
            stop_bits: serial::StopBits::Stop1,
            flow_control: serial::FlowControl::FlowNone
        }).expect("Unable to configure serial port");

        let transmitter = STM32Transmitter {
            serial: Box::new(serial)
        };

        transmitter
    }
}

impl Transmitter for STM32Transmitter {
    fn send(&mut self, gen: Generator) {

        thread::sleep(time::Duration::from_millis(1));

        for word in gen {
            let bytes = [((word & 0xff000000) >> 24) as u8,
                         ((word & 0x00ff0000) >> 16) as u8,
                         ((word & 0x0000ff00) >> 8) as u8,
                         ((word & 0x000000ff)) as u8];

            if (*self.serial).write(&bytes).is_err() {
                error!("Unable to write data to the serial port");
                return;
            }
        }
        // Send End of Transmission packet
        let eot = [0x17 as u8];
        if (*self.serial).write(&eot).is_err() {
            error!("Unable to send end of transmission byte");
            return;
        }
    }
}
