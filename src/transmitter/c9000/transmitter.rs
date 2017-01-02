use raspi::{Gpio, Pin, Direction, Model};
use std::{thread, time};
use std::ops::DerefMut;
use serial;

use pocsag::Generator;

pub struct Transmitter {
    reset_pin: Pin,
    ptt_pin: Pin,
    send_pin: Pin,
    serial: Box<serial::SerialPort>
}

impl Transmitter  {
    pub fn new() -> Transmitter {
        info!("Initializing C9000 transmitter.");
        info!("Detected {}", Model::get());
        let serial = serial::open("/dev/ttyAMA0").expect("Unable to open serial port");
        let gpio = Gpio::new().expect("Failed to map GPIO");

        let transmitter = Transmitter {
            reset_pin: gpio.pin(17, Direction::Output),
            ptt_pin: gpio.pin(27, Direction::Output),
            send_pin: gpio.pin(22, Direction::Output),
            serial: Box::new(serial)
        };

        transmitter
    }

    pub fn run(&mut self) {

    }
}

impl ::transmitter::Transmitter for Transmitter {
    fn send(&mut self, gen: Generator) {
        info!("Sending data...");

        self.ptt_pin.set_high();

        thread::sleep(time::Duration::from_millis(1));

        for word in gen {
            while !self.send_pin.read() {
                time::Duration::from_millis(1);
            }

            //if self.serial.deref_mut().write(&[byte]).is_err() {
            //    error!("Unable to write data to the serial port");
            //}
        }

        self.ptt_pin.set_low();

        info!("Data sent.");
    }
}
