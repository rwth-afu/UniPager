use pocsag::{Message, MessageFunc, Encoding, encoding};

/// Preamble length in number of 32-bit codewords
const PREAMBLE_LENGTH: u8 = 18;

const SYNC_WORD: u32 = 0x7CD215D8;
const IDLE_WORD: u32 = 0x7A89C197;

#[derive(Clone, Copy, Debug)]
enum State {
    Preamble,
    AddressWord,
    MessageWord(usize, Encoding),
    Completed
}

pub struct Generator {
    state: State,
    messages: Vec<Message>,
    message: Option<Message>,
    codewords: u8
}

impl Generator {
    pub fn new(messages: Vec<Message>) -> Generator {
        Generator {
            state: State::Preamble,
            messages: messages,
            message: None,
            codewords: PREAMBLE_LENGTH
        }
    }

    fn next_message(&mut self) -> State {
        self.message = self.messages.pop();
        match self.message {
            Some(_) => State::AddressWord,
            None => State::Completed
        }
    }
}

fn crc(codeword: u32) -> u32 {
    let mut crc = codeword;
    for i in 0..21 {
        if (crc & (0x80000000 >> i)) != 0 {
            crc ^= 0xED200000 >> i;
        }
    }
    codeword | crc
}

fn parity(codeword: u32) -> u32 {
    let mut parity = codeword ^ (codeword >> 1);
    parity ^= parity >> 2;
    parity ^= parity >> 4;
    parity ^= parity >> 8;
    parity ^= parity >> 16;
    codeword | (parity & 1)
}

impl Iterator for Generator {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        debug!("({}, {:?})", self.codewords, self.state);
        match (self.codewords, self.state) {
            (0, State::Completed) => None,
            (0, State::Preamble) => {
                self.codewords = 16;
                self.state = self.next_message();
                Some(SYNC_WORD)
            }
            (0, _) => {
                self.codewords = 16;
                Some(SYNC_WORD)
            },
            (_, State::Preamble) => { self.codewords -= 1; Some(0xAAAAAAAA) },
            (codeword, State::AddressWord) => {
                let &Message { addr, func, .. } = self.message.as_ref().unwrap();
                self.codewords -= 1;

                if ((addr & 0b111) << 1) as u8 == 16 - codeword {
                    self.state = match func {
                        MessageFunc::Tone =>
                            self.next_message(),
                        MessageFunc::Numeric =>
                            State::MessageWord(0, encoding::NUMERIC),
                        MessageFunc::AlphaNum =>
                            State::MessageWord(0, encoding::ALPHANUM),
                        MessageFunc::Activation =>
                            State::MessageWord(0, encoding::ALPHANUM)
                    };

                    let addr = (addr & 0x001ffff8) << 10;
                    let func = (func as u32 & 0b11) << 11;
                    Some(parity(crc(addr | func)))
                }
                else {
                    Some(IDLE_WORD)
                }
            },
            (_, State::MessageWord(pos, encoding)) => {
                self.codewords -= 1;
                let mut pos = pos;
                let mut codeword: u32 = 0;

                let completed = {
                    let mut bytes = self.message.as_ref().unwrap().data.bytes();

                    let mut sym = bytes
                        .nth(pos / encoding.bits)
                        .map(encoding.encode)
                        .unwrap_or(encoding.trailing) >> pos % encoding.bits;

                    for _ in 0..20 {
                        codeword <<= 1;
                        pos += 1;

                        codeword |= (sym & 1) as u32;

                        if pos % encoding.bits == 0 {
                            sym = bytes.next()
                                .map(encoding.encode)
                                .unwrap_or(encoding.trailing);
                        }
                        else {
                            sym >>= 1;
                        }
                    }

                    bytes.next().is_none()
                };

                self.state = match completed {
                    true => self.next_message(),
                    false => State::MessageWord(pos, encoding)
                };

                // TODO: ensure that an trailing IDLE, SYNC or ADDR word is sent

                Some(parity(crc(0x80000000 | (codeword << 11))))
            },
            (_, State::Completed) => {
                self.codewords -= 1;
                Some(IDLE_WORD)
            }
        }
    }
}
