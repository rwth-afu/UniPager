use std::fs::File;
use std::io::Read;
use std::ops::BitAnd;
use std::u32;
use std::fmt;

pub enum Model {
    V1A,
    V1B { rev: u8 },
    V1Aplus,
    V1Bplus,
    V2B,
    V3B,
    V3Bplus,
    V4B,
    Pi400,
    Zero,
    ZeroW,
    OrangePi,
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
            .unwrap_or(0x0)
            .bitand(0x00ffffff);

        let hardware = cpuinfo.split('\n')
            .filter(|line| line.starts_with("Hardware")).next()
            .and_then(|line| line.split(':').nth(1))
            .map(str::trim);

        // See https://www.raspberrypi.org/documentation/hardware/raspberrypi/revision-codes/README.md
        match revision {
            0x2..=0x3 => Model::V1B { rev: 1 },
            0x4..=0x6 | 0xd..=0x0f => Model::V1B { rev: 2 },
            0x7..=0x9 => Model::V1A,
            0x12 | 0x15 | 0x900021 => Model::V1Aplus,
            0x10 | 0x13 | 0x900032 => Model::V1Bplus,
            0xA01040 | 0xA01041 => Model::V2B,
            0xA21041 => Model::V2B,
            0xA22042 | 0xA02042 => Model::V2B, // with BCM2837
            0x900092 | 0x900093 | 0x920093 => Model::Zero,
            0x9000C1 => Model::ZeroW,
            0xA02082 | 0xA22082 | 0xA32082 => Model::V3B,
            0xA020D3 => Model::V3Bplus,
            0xA03111 | 0xB03111..=0xB03114 | 0xC03111..=0xC03114 | 0xD03114 => Model::V4B,
            0xC03130 => Model::Pi400,
            _ => match hardware {
                Some("Allwinner sun8i Family") => Model::OrangePi,
                Some("sun8i") => Model::OrangePi,
                _ => Model::Unknown
            }
        }
    }

    pub fn gpio_base(&self) -> Option<u32> {
        match self {
            &Model::V1A | &Model::V1B { rev: _ } |
            &Model::V1Aplus | &Model::V1Bplus
                => Some(0x20200000),
            &Model::V2B => Some(0x3F200000),
            &Model::V3B | &Model::V3Bplus => Some(0x3F200000),
            &Model::V4B | &Model::Pi400 => Some(0xFE200000),
            &Model::Zero | &Model::ZeroW => Some(0x20200000),
            &Model::OrangePi => None,
            &Model::Unknown => None
        }
    }

    pub fn pin_mapping(&self) -> Option<Vec<usize>> {
        match self {
            &Model::V1B { rev: 1 } =>
                Some(vec![17, 18, 21, 22, 23, 24, 25, 4,
                          0, 1, 8, 7, 10, 9, 11, 14, 15]),
            &Model::V1A | &Model::V1B { rev: _ } | &Model::V2B |
            &Model::V1Aplus | &Model::V1Bplus |
            &Model::Zero | &Model::ZeroW =>
                Some(vec![17, 18, 27, 22, 23, 24, 25, 4,
                          2, 3, 8, 7, 10, 9, 11, 14, 15]),
            &Model::V3B | &Model::V3Bplus | &Model::V4B | &Model::Pi400 =>
                Some(vec![17, 18, 27, 22, 23, 24, 25, 4,
                     2, 3, 8, 7, 10, 9, 11, 14, 15,
                     0, 0, 0, 0, 5, 6, 13, 19, 26,
                     12, 16, 20, 21, 0, 1]),
            &Model::OrangePi => None,
            &Model::Unknown => None
        }
    }

    pub fn serial_port(&self) -> &'static str {
        "/dev/ttyAMA0"
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Model::V1A => write!(f, "Raspberry Pi 1 Model A"),
            &Model::V1B { rev } => write!(f, "Raspberry Pi 1 Model B Rev. {}", rev),
            &Model::V1Aplus => write!(f, "Raspberry Pi 1 Model A+"),
            &Model::V1Bplus => write!(f, "Raspberry Pi 1 Model B+"),
            &Model::V2B => write!(f, "Raspberry Pi 2 Model B"),
            &Model::V3B => write!(f, "Raspberry Pi 3 Model B"),
            &Model::V3Bplus => write!(f, "Raspberry Pi 3 Model B+"),
            &Model::V4B => write!(f, "Raspberry Pi 4 Model B"),
            &Model::Pi400 => write!(f, "Raspberry Pi 400"),
            &Model::Zero => write!(f, "Raspberry Pi Zero"),
            &Model::ZeroW => write!(f, "Raspberry Pi Zero W"),
            &Model::OrangePi => write!(f, "Orange Pi"),
            &Model::Unknown => write!(f, "Unknown Device")
        }
    }
}
