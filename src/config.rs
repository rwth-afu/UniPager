use std::fs::File;
use std::io::{Read, Write};
use std::fmt;
use serde_json;

const CONFIG_FILE: &'static str = "config.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct C9000Config {
    pub baudrate: u32
}

impl Default for C9000Config {
    fn default() -> C9000Config {
        C9000Config {
            baudrate: 38400
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RaspagerConfig {
    pub freq: u32,
    pub freq_corr: i16,
    pub pa_output_level: u8
}

impl Default for RaspagerConfig {
    fn default() -> RaspagerConfig {
        RaspagerConfig {
            freq: 439987500,
            freq_corr: 0,
            pa_output_level: 30
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RFM69Config {
    pub port: String
}

impl Default for RFM69Config {
    fn default() -> RFM69Config {
        RFM69Config {
            port: String::from("/dev/ttyUSB0")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioConfig {
    #[serde(default)]
    pub device: String,
    pub level: u8,
    pub inverted: bool,
    #[serde(default)]
    pub tx_delay: usize
}

impl Default for AudioConfig {
    fn default() -> AudioConfig {
        AudioConfig {
            device: String::from("default"),
            level: 127,
            inverted: false,
            tx_delay: 0
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PttMethod {
    Gpio,
    SerialDtr,
    SerialRts
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PttConfig {
    pub method: PttMethod,
    pub inverted: bool,
    pub gpio_pin: usize,
    pub serial_port: String
}

impl Default for PttConfig {
    fn default() -> PttConfig {
        PttConfig {
            method: PttMethod::Gpio,
            inverted: false,
            gpio_pin: 0,
            serial_port: String::from("/dev/ttyS0")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MasterConfig {
    pub server: String,
    pub port: u16,
    #[serde(default)]
    pub call: String,
    #[serde(default)]
    pub auth: String
}

impl Default for MasterConfig {
    fn default() -> MasterConfig {
        MasterConfig {
            server: String::from("dapnet.afu.rwth-aachen.de"),
            port: 43434,
            call: String::from(""),
            auth: String::from("")
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Transmitter {
    Dummy,
    Audio,
    C9000,
    Raspager,
    RFM69 
}

impl Default for Transmitter {
    fn default() -> Transmitter {
        Transmitter::Dummy
    }
}

impl fmt::Display for Transmitter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            Transmitter::Dummy => "Dummy",
            Transmitter::Audio => "Audio",
            Transmitter::C9000 => "C9000",
            Transmitter::Raspager => "RaspagerV1",
            Transmitter::RFM69 => "RFM69"
        };
        write!(f, "{}", name)
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub master: MasterConfig,
    pub transmitter: Transmitter,
    #[serde(default)]
    pub ptt: PttConfig,
    #[serde(default)]
    pub raspager: RaspagerConfig,
    #[serde(default)]
    pub c9000: C9000Config,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub rfm69: RFM69Config,
}

impl Config {
    pub fn load() -> Config {
         match File::open(CONFIG_FILE) {
             Ok(mut file) => {
                 let mut data = String::new();
                 file.read_to_string(&mut data)
                     .expect("Failed to read config file");

                 if let Ok(config) = serde_json::from_str(&data) {
                     config
                 }
                 else {
                     error!("Failed to parse config file. Using default.");
                     Config::default()
                 }
             },
             Err(_) => {
                 info!("Creating config file from default config.");
                 let config = Config::default();
                 config.save();
                 config
             }
        }
    }

    pub fn save(&self) {
        let data = serde_json::to_vec_pretty(self).unwrap();

        let mut new_file = File::create(CONFIG_FILE)
            .expect("Couldn't create config file");

        new_file.write_all(data.as_slice())
            .expect("Couldn't write to config file");
    }
}
