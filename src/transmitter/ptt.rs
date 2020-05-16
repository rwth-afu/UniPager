use crate::config::{PttConfig, PttMethod};
use raspi::{Direction, Gpio, Model, Pin};
use serial;
use std::ffi::CString;

pub enum Ptt {
    Gpio { pin: Box<dyn Pin>, inverted: bool },
    SerialDtr {
        port: Box<dyn serial::SerialPort>,
        inverted: bool
    },
    SerialRts {
        port: Box<dyn serial::SerialPort>,
        inverted: bool
    },
    HidRaw {
        device: Box<hidapi::HidDevice>,
        gpio: u8,
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
                info!("Using device {}", &*config.hidraw_device);
                let path = CString::new(&*config.hidraw_device).unwrap();
                for device in api.devices() {
                    if device.path == path {
                        if device.vendor_id == 0x0d8c && (device.product_id == 0x013c || device.product_id == 0x000c) {
                            info!("Found CM108 device {:#06x}/{:#06x}", device.vendor_id, device.product_id);
                        } else {
                            error!("Unsupported device {:#06x}/{:#06x}!", device.vendor_id, device.product_id);
                        }
                    }
                }
                let cm108device = api.open_path(&path).expect(
                    "Unable to open HIDraw device"
                );
                let mut string = "Device data: manufacturer \"".to_string();
                let manufacturer = cm108device.get_manufacturer_string().unwrap();
                match manufacturer {
                    Some(x) => string.push_str(&x.trim()),
                    None    => string.push_str("n/a"),
                }
                string.push_str("\", product \"");
                let product = cm108device.get_product_string().unwrap();
                match product {
                    Some(x) => string.push_str(&x.trim()),
                    None    => string.push_str("n/a"),
                }
                info!("{}\"", string);

                let gpio = config.hidraw_gpio_pin;
                let mut pin = 0x00;
                let mut string = "PTT GPIO pin: ".to_string();
                match gpio {
                    1 =>  {
                        pin = 0x01;
                        string.push_str("1");
                    }
                    2 => {
                        pin = 0x02;
                        string.push_str("2");
                    }
                    3 => {
                        pin = 0x04;
                        string.push_str("3");
                    }
                    4 => {
                        pin = 0x08;
                        string.push_str("4");
                    }
                    _ => error!("Invalid GPIO pin!")
                }
                info!("{}", string);

                Ptt::HidRaw {
                    device: Box::new(cm108device),
                    gpio: pin,
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
                gpio,
                inverted
            } => {
                if status != inverted {
                    let buf = [0x00, 0x00, gpio, gpio, 0x00];
                    device.write(&buf).expect(
                        "Error writing hidraw interface"
                    );
                } else {
                    let buf = [0x00, 0x00, 0x00, gpio, 0x00];
                    device.write(&buf).expect(
                        "Error writing hidraw interface"
                    );
                }
            }
        }
    }
}
