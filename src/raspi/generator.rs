use sysfs_gpio::{Pin, Direction};
use std::{thread, time};
use std::ops::DerefMut;
//use std::cell::RefCell;
use serial;

pub struct Generator {
    reset_pin: Pin,
    ptt_pin: Pin,
    send_pin: Pin,
    serial: Box<serial::SerialPort>
}

impl Generator  {
    pub fn new() -> Generator {
        let serial = serial::open("/dev/ttyAMA0").expect("Unable to open serial port");

        let generator = Generator {
            reset_pin: Pin::new(17),
            ptt_pin: Pin::new(27),
            send_pin: Pin::new(22),
            serial: Box::new(serial)
        };

        generator.ptt_pin.export()
            .expect("Unable to export PTT pin");
        generator.ptt_pin.set_direction(Direction::High)
            .expect("Unable to set PTT pin as high output");

        generator.send_pin.export()
            .expect("Unable to export SEND pin");
        generator.send_pin.set_direction(Direction::In)
            .expect("Unable to set SEND pin as input");

        generator.reset_pin.export()
            .expect("Unable to export RESET pin");
        generator.reset_pin.set_direction(Direction::High)
            .expect("Unable to set RESET pin as high output");

        generator
    }

    pub fn run(&mut self) {

    }

    pub fn send(&mut self, data: &str) {
        info!("Sending data...");

        if self.ptt_pin.set_value(1).is_err() {
            error!("Unable to set PTT pin to high");
        }

        thread::sleep(time::Duration::from_millis(1));

        for byte in data.bytes() {
            while self.send_pin.get_value().unwrap_or(0) == 0 {
                time::Duration::from_millis(1);
            }

            if self.serial.deref_mut().write(&[byte]).is_err() {
                error!("Unable to write data to the serial port");
            }
        }

        if self.ptt_pin.set_value(0).is_err() {
            error!("Unable to set PTT pin to low");
        }

        info!("Data sent.");
    }
}
