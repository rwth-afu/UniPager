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
    status_led_pin: Pin,
    connected_led_pin: Pin,
    serial: Box<serial::SerialPort>
}

impl C9000Transmitter  {
    pub fn new(_: &Config) -> C9000Transmitter {
        info!("Initializing C9000 transmitter...");

        let model = Model::get();
        info!("Detected {}", model);

        let mut serial = serial::open(model.serial_port())
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
            status_led_pin: gpio.pin(10, Direction::Output),
            connected_led_pin: gpio.pin(11, Direction::Output),
            serial: Box::new(serial)
        };

        transmitter.reset_pin.set_high();
        transmitter.status_led_pin.set_high();
        transmitter.connected_led_pin.set_high();

        transmitter
    }
}

impl Transmitter for C9000Transmitter {
    fn send(&mut self, gen: Generator) {
        self.ptt_pin.set_high();

        thread::sleep(time::Duration::from_millis(1));

        for (i, word) in gen.enumerate() {
            if i % 40 == 0 {
                if (*self.serial).flush().is_err() {
                    error!("Unable to flush serial port");
                }

                while !self.send_pin.read() {
                    thread::sleep(time::Duration::from_millis(1));
                }
            }

            let bytes = [((word & 0xff000000) >> 24) as u8,
                         ((word & 0x00ff0000) >> 16) as u8,
                         ((word & 0x0000ff00) >> 8) as u8,
                         ((word & 0x000000ff)) as u8];

            if (*self.serial).write_all(&bytes).is_err() {
                error!("Unable to write data to the serial port");
                self.ptt_pin.set_low();
                return;
            }
        }

        if (*self.serial).flush().is_err() {
            error!("Unable to flush serial port");
        }

        self.ptt_pin.set_low();
    }
}
