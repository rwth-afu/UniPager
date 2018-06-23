use config::{PttConfig, PttMethod};
use raspi::{Direction, Gpio, Model, Pin};
use serial;

pub enum Ptt {
    Gpio { pin: Box<Pin>, inverted: bool },
    SerialDtr {
        port: Box<serial::SerialPort>,
        inverted: bool
    },
    SerialRts {
        port: Box<serial::SerialPort>,
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
        }
    }
}
