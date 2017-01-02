use std::fs::File;
use std::io::Read;
use std::u32;
use std::fmt;

pub enum Model {
    V1A,
    V1B,
    V2B,
    V3B,
    Zero,
    Unknown
}

impl Model {
    pub fn get() -> Model {
        let mut file = match File::open("/proc/cpuinfo") {
            Ok(file) => file,
            Err(_) => return Model::Unknown
        };

        let mut cpuinfo = String::new();
        if file.read_to_string(&mut cpuinfo).is_err() {
            return Model::Unknown;
        }

        let revision = cpuinfo.split('\n')
            .filter(|line| line.starts_with("Revision")).next()
            .and_then(|line| line.split(':').nth(1))
            .map(str::trim)
            .and_then(|res| u32::from_str_radix(res, 16).ok())
            .unwrap_or(0x0);

        match revision {
            0x2...0x6 | 0x13 | 0xd...0x10 => Model::V1B,
            0x7...0x9 | 0x12 | 0x15 => Model::V1A,
            0xA01040 | 0xA01041 => Model::V2B,
            0xA22042 => Model::V2B, // with BCM2837
            0x900021 => Model::V1A,
            0x900092 | 0x900093 | 0x920093 => Model::Zero,
            0xA02082 | 0xA22082 => Model::V3B,
            _ => Model::Unknown
        }
    }

    pub fn gpio_base(&self) -> Option<u32> {
        match self {
            &Model::V1A => Some(0x20200000),
            &Model::V1B => Some(0x20200000),
            &Model::V2B => Some(0x3F200000),
            &Model::V3B => Some(0x3F200000),
            &Model::Zero => Some(0x20200000),
            &Model::Unknown => None
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Model::V1A => write!(f, "Raspberry Pi 1 Model A"),
            &Model::V1B => write!(f, "Raspberry Pi 1 Model B"),
            &Model::V2B => write!(f, "Raspberry Pi 2 Model B"),
            &Model::V3B => write!(f, "Raspberry Pi 3 Model B"),
            &Model::Zero => write!(f, "Raspberry Pi Zero"),
            &Model::Unknown => write!(f, "Unknown Raspberry Pi")
        }
    }
}
