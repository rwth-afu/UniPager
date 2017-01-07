use std::{thread, time};
use serial::{self, SerialPort};
use raspi::{Gpio, Pin, Direction, Model};

use config::Config;
use pocsag::Generator;
use transmitter::Transmitter;

pub struct C9000Transmitter {
    reset_pin: Pin,
    ptt_pin: Pin,
    send_pin: Pin,
    serial: Box<serial::SerialPort>
}

impl C9000Transmitter  {
    pub fn new(_: &Config) -> C9000Transmitter {
        info!("Initializing C9000 transmitter...");
        info!("Detected {}", Model::get());

        let mut serial = serial::open("/dev/ttyAMA0")
            .expect("Unable to open serial port");

        serial.configure(&serial::PortSettings {
            baud_rate: serial::BaudRate::Baud38400,
            char_size: serial::CharSize::Bits8,
            parity: serial::Parity::ParityNone,
            stop_bits: serial::StopBits::Stop1,
            flow_control: serial::FlowControl::FlowNone
        }).expect("Unable to configure serial port");

        let gpio = Gpio::new().expect("Failed to map GPIO");

        let transmitter = C9000Transmitter {
            reset_pin: gpio.pin(0, Direction::Output),
            ptt_pin: gpio.pin(2, Direction::Output),
            send_pin: gpio.pin(3, Direction::Input),
            serial: Box::new(serial)
        };

        transmitter.reset_pin.set_high();

        transmitter
    }
}

impl Transmitter for C9000Transmitter {
    fn send(&mut self, gen: Generator) {
        info!("Sending data...");

        self.ptt_pin.set_high();

        thread::sleep(time::Duration::from_millis(1));

        for (i, word) in gen.enumerate() {
            while (i % 40 == 0) && !self.send_pin.read() {
                thread::sleep(time::Duration::from_millis(1));
            }

            let bytes = [(word & 0xff000000 >> 24) as u8,
                         (word & 0x00ff0000 >> 16) as u8,
                         (word & 0x0000ff00 >> 8) as u8,
                         (word & 0x000000ff) as u8];

            if (*self.serial).write(&bytes).is_err() {
                error!("Unable to write data to the serial port");
                self.ptt_pin.set_low();
                return;
            }
        }

        self.ptt_pin.set_low();

        info!("Data sent.");
    }
}
