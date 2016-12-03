use message::Message;

#[derive(Debug)]
enum State {
    Preamble(u8),
    Message(Message),
    SyncWord(u8),
    IdleWord(u8),
    Completed
}

pub struct Generator {
    state: State,
    messages: Vec<Message>
}

impl Generator {
    pub fn new(messages: Vec<Message>) -> Generator {
        Generator {
            state: State::Preamble(72),
            messages: messages
        }
    }
}

impl Iterator for Generator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        match self.state {
            State::Preamble(0) => { self.state = State::SyncWord(3); Some(0b10101010) },
            State::Preamble(rem) => { self.state = State::Preamble(rem - 1); Some(0b10101010) },
            State::SyncWord(3) => { self.state = State::SyncWord(2); Some(0x7C) },
            State::SyncWord(2) => { self.state = State::SyncWord(1); Some(0xD2) },
            State::SyncWord(1) => { self.state = State::SyncWord(0); Some(0x15) },
            State::SyncWord(0) => { self.state = State::IdleWord(3); Some(0xD8) },
            State::IdleWord(3) => { self.state = State::IdleWord(2); Some(0x7A) },
            State::IdleWord(2) => { self.state = State::IdleWord(1); Some(0x89) },
            State::IdleWord(1) => { self.state = State::IdleWord(0); Some(0xC1) },
            State::IdleWord(0) => { self.state = State::Completed;   Some(0x97) },
            State::Completed => { None },
            ref invalid => {
                panic!("Invalid Generator State: {:?}", invalid)
            }
        }
    }
}

#[test]
pub fn test_generator() {
    let messages = vec![];
    let generator = Generator::new(messages);
    for byte in generator {
        println!("{:08b}", byte);
    }
}
