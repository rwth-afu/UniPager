use std::fs::File;
use std::io::{Read, Write};
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
pub struct MasterConfig {
    pub server: String,
    pub port: u16
}

impl Default for MasterConfig {
    fn default() -> MasterConfig {
        MasterConfig {
            server: String::from("44.225.164.2"),
            port: 1337
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum Transmitter {
    Dummy,
    Audio,
    C9000,
    Raspager
}

impl Default for Transmitter {
    fn default() -> Transmitter {
        Transmitter::Dummy
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub master: MasterConfig,
    pub transmitter: Transmitter,
    pub raspager: RaspagerConfig,
    pub c9000: C9000Config
}

impl Config {
    pub fn load() -> Config {
         match File::open(CONFIG_FILE) {
             Ok(mut file) => {
                 let mut data = String::new();
                 file.read_to_string(&mut data)
                     .expect("Failed to read config file");

                 serde_json::from_str(&data)
                     .unwrap_or(Config::default())
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
        let data = serde_json::to_vec(self).unwrap();

        let mut new_file = File::create(CONFIG_FILE)
            .expect("Couldn't create config file");

        new_file.write_all(data.as_slice())
            .expect("Couldn't write to config file");
    }
}
