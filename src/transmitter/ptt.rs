use config::{PttConfig, PttMethod};
use raspi::{Direction, Gpio, Model, Pin};
use serial;
use std::ffi::CString;

pub enum Ptt {
    Gpio { pin: Box<Pin>, inverted: bool },
    SerialDtr {
        port: Box<serial::SerialPort>,
        inverted: bool
    },
    SerialRts {
        port: Box<serial::SerialPort>,
        inverted: bool
    },
    HidRaw {
        device: Box<hidapi::HidDevice>,
        inverted: bool
    }
}

impl Ptt {
    pub fn from_config(config: &PttConfig) -> Ptt {
        match config.method {
            PttMethod::Gpio => {
                info!("Detected {}", Model::get());
                let gpio = Gpio::new().expect("Failed to map GPIO");

                Ptt::Gpio {
                    pin: gpio.pin(config.gpio_pin, Direction::Output),
                    inverted: config.inverted
                }
            }
            PttMethod::SerialDtr => {
                let port = serial::open(&*config.serial_port).expect(
                    "Unable to open serial port"
                );

                Ptt::SerialDtr {
                    port: Box::new(port),
                    inverted: config.inverted
                }
            }

            PttMethod::SerialRts => {
                let port = serial::open(&*config.serial_port).expect(
                    "Unable to open serial port"
                );

                Ptt::SerialRts {
                    port: Box::new(port),
                    inverted: config.inverted
                }
            }

            PttMethod::HidRaw => {
                let api = hidapi::HidApi::new().expect(
                    "Unable to initialize HID API"
                );
                let path = CString::new(&*config.hidraw_device).unwrap();
                let device = api.open_path(&path).expect(
                    "Unable to open HIDraw device"
                );

                let mut string = "Found HIDraw device, manufacturer \"".to_string();
                let manufacturer = device.get_manufacturer_string().unwrap();
                match manufacturer {
                    Some(x) => string.push_str(&x.trim()),
                    None    => string.push_str("n/a"),
                }
                string.push_str("\", product \"");
                let product = device.get_product_string().unwrap();
                match product {
                    Some(x) => string.push_str(&x.trim()),
                    None    => string.push_str("n/a"),
                }
                info!("{}\"", string);

                Ptt::HidRaw {
                    device: Box::new(device),
                    inverted: config.inverted
                }
            }
        }
    }

    pub fn set(&mut self, status: bool) {
        match *self {
            Ptt::Gpio { ref pin, inverted } => {
                pin.set(status != inverted);
            }
            Ptt::SerialDtr {
                ref mut port,
                inverted
            } => {
                port.set_dtr(status != inverted).expect(
                    "Error setting DTR pin"
                );
            }
            Ptt::SerialRts {
                ref mut port,
                inverted
            } => {
                port.set_rts(status != inverted).expect(
                    "Error setting RTS pin"
                );
            }
            Ptt::HidRaw {
                ref mut device,
                inverted
            } => {
                if status != inverted {
                    // Write data to device
                    let buf = [0x00, 0x00, 0x04, 0x04, 0x00];
                    device.write(&buf).expect(
                        "Error writing hidraw interface"
                    );
                } else {
                    let buf = [0x00, 0x00, 0x00, 0x04, 0x00];
                    device.write(&buf).expect(
                        "Error writing hidraw interface"
                    );
                }
            }
        }
    }
}
