use serde_json;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};

const CONFIG_FILE: &'static str = "config.json";

fn default_fallback_servers() -> Vec<(String, u16)> {
    [
        ("funkruf.db0del.db0sda.ampr.org", 43434),
        ("dapnet.db0fa.ampr.org", 43434),
        ("pocsag2.db0nms.ampr.org", 43434),
        ("dapnet.db0sda.ampr.org", 43434),
        ("dapnet.db0vvs.ampr.org", 43434),
        ("dapnet.di0han.ampr.org", 43434),
        ("on3dhc.db0sda.ampr.org", 43434),
        ("dapnet.afu.rwth-aachen.de", 43434),
        ("db0dbn.ig-funk-siebengebirge.de", 43434)
    ]
        .iter()
        .map(|&(ref host, port)| (host.to_string(), port))
        .collect()
}

fn default_mod_deviation() -> u16 { 13 }

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct C9000Config {
    pub baudrate: u32,
    pub dummy_enabled: bool,
    pub dummy_port: String,
    pub dummy_pa_output_level: u8
}

impl Default for C9000Config {
    fn default() -> C9000Config {
        C9000Config {
            baudrate: 38400,
            dummy_enabled: false,
            dummy_port: String::from("/dev/ttyUSB0"),
            dummy_pa_output_level: 0
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RaspagerConfig {
    pub freq: u32,
    pub freq_corr: i16,
    pub pa_output_level: u8,
    pub mod_deviation: u16,
}

impl Default for RaspagerConfig {
    fn default() -> RaspagerConfig {
        RaspagerConfig {
            freq: 439987500,
            freq_corr: 0,
            pa_output_level: 63,
            mod_deviation: default_mod_deviation()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RFM69Config {
    pub port: String
}

impl Default for RFM69Config {
    fn default() -> RFM69Config {
        RFM69Config { port: String::from("/dev/ttyUSB0") }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
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
    SerialRts,
    HidRaw
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct PttConfig {
    pub method: PttMethod,
    pub inverted: bool,
    pub gpio_pin: usize,
    pub serial_port: String,
    pub hidraw_device: String
}

impl Default for PttConfig {
    fn default() -> PttConfig {
        PttConfig {
            method: PttMethod::Gpio,
            inverted: false,
            gpio_pin: 0,
            serial_port: String::from("/dev/ttyS0"),
            hidraw_device: String::from("/dev/hidraw0")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MasterConfig {
    pub server: String,
    pub port: u16,
    pub call: String,
    pub auth: String,
    pub fallback: Vec<(String, u16)>
}

impl Default for MasterConfig {
    fn default() -> MasterConfig {
        MasterConfig {
            server: String::from("dapnet.afu.rwth-aachen.de"),
            port: 43434,
            call: String::from(""),
            auth: String::from(""),
            fallback: default_fallback_servers()
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Transmitter {
    Dummy,
    Audio,
    C9000,
    Raspager,
    Raspager2,
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
            Transmitter::Raspager => "Raspager1",
            Transmitter::Raspager2 => "Raspager2",
            Transmitter::RFM69 => "RFM69",
        };
        write!(f, "{}", name)
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub master: MasterConfig,
    pub transmitter: Transmitter,
    pub ptt: PttConfig,
    pub raspager: RaspagerConfig,
    pub c9000: C9000Config,
    pub audio: AudioConfig,
    pub rfm69: RFM69Config
}

impl Config {
    pub fn load() -> Config {
        match File::open(CONFIG_FILE) {
            Ok(mut file) => {
                let mut data = String::new();
                file.read_to_string(&mut data).expect(
                    "Failed to read config file"
                );

                if let Ok(config) = serde_json::from_str(&data) {
                    config
                } else {
                    error!("Failed to parse config file. Using default.");
                    Config::default()
                }
            }
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

        let mut new_file = File::create(CONFIG_FILE).expect(
            "Couldn't create config file"
        );

        new_file.write_all(data.as_slice()).expect(
            "Couldn't write to config file"
        );
    }
}
