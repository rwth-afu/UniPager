#[derive(Debug, Copy, Clone)]
pub struct Encoding {
    pub encode: fn(u8) -> u8,
    pub bits: usize,
    pub trailing: u8
}

fn encode_alphanum(byte: u8) -> u8 {
    match byte {
        0..=127 => byte,
        _ => 0x3F,
    }
}

fn encode_numeric(byte: u8) -> u8 {
    match byte as char {
        '0' => 0x0,
        '1' => 0x1,
        '2' => 0x2,
        '3' => 0x3,
        '4' => 0x4,
        '5' => 0x5,
        '6' => 0x6,
        '7' => 0x7,
        '8' => 0x8,
        '9' => 0x9,
        '*' => 0xA,
        'U' => 0xB,
        ' ' => 0xC,
        '-' => 0xD,
        ')' => 0xE,
        '(' => 0xF,
        _ => 0xC,
    }
}

pub const ALPHANUM: Encoding = Encoding {
    encode: encode_alphanum,
    bits: 7,
    trailing: 0x0
};
pub const NUMERIC: Encoding = Encoding {
    encode: encode_numeric,
    bits: 4,
    trailing: 0xc
};
