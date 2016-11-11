use std::fs::File;
use std::io::{Read, Write};
use toml;

#[derive(Debug)]
pub struct Config {
    data: toml::Value,
    filename: String
}

impl Config {
    pub fn load() -> Config {
        let filename = "config.toml";

        let file = File::open(filename);

        let mut file = if file.is_err() {
            let mut new_file = File::create(filename).expect("Couldn't create config.toml");

            let default_config = include_bytes!("config.toml.default");
            new_file.write_all(default_config).expect("Couldn't write to config.toml");
            drop(new_file);

            File::open(filename).expect("Couldn't open config.toml")
        }
        else {
            file.expect("Couldn't open config.toml")
        };

        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();

        let mut parser = toml::Parser::new(&s);
        let result = parser.parse().expect("Invalid config.toml");

        Config { filename: String::from(filename), data: toml::Value::Table(result) }
    }

    pub fn get<'a>(&'a self, path: &'a str) -> Option<&'a toml::Value> {
        self.data.lookup(path)
    }

    pub fn get_str<'a>(&'a self, path: &'a str) -> Option<&'a str> {
        self.get(path).and_then(|v| v.as_str()).and_then(|v| if v.is_empty() { None } else { Some(v) })
    }

    pub fn get_int(&self, path: &str) -> Option<i64> {
        self.get(path).and_then(|v| v.as_integer())
    }

    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path).and_then(|v| v.as_bool())
    }
}
