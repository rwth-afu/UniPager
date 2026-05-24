use crate::message::{MessageProvider, ProtocolMessage};
use crate::pocsag::{Encoding, Message, MessageType, encoding};

/// Preamble length in number of 32-bit codewords
pub const PREAMBLE_LENGTH: u8 = 18;

const SYNC_WORD: u32 = 0x7CD215D8;
const IDLE_WORD: u32 = 0x7A89C197;

#[derive(Clone, Copy, Debug)]
enum State {
    Preamble,
    AddressWord,
    MessageWord(usize, Encoding),
    // Fill the rest of the current batch with idles, then transition to
    // Terminator on the batch boundary.
    Completed,
    // Emit idle words in a final batch following a trailing sync word, then
    // end the iterator. Guarantees at least one full sync+idle batch follows
    // the last message word so receivers see a clear end-of-message and the
    // carrier stays up long enough to commit it.
    Terminator
}

/// POCSAG Generator
///
/// Generates 32-bit POCSAG codewords from a Message vector.
pub struct Generator<'a> {
    // Current state of the state machine
    state: State,
    // Message source
    messages: &'a mut (dyn MessageProvider + 'a),
    // Current message being sent
    message: Option<Message>,
    // Number of codewords left in current batch
    codewords: u8,
    // Number of codewords generated
    count: usize
}

impl<'a> Generator<'a> {
    /// Create a new Generator
    pub fn new(messages: &'a mut dyn MessageProvider, first_msg: Message)
        -> Generator<'a> {
        Generator {
            state: State::Preamble,
            messages,
            message: Some(first_msg),
            codewords: PREAMBLE_LENGTH,
            count: 0
        }
    }

    // Get the next message and return the matching state.
    fn next_message(&mut self) -> State {
        let message = self.messages.next(self.count - 1).map(|msg| msg.message);
        self.message = match message {
            Some(ProtocolMessage::Pocsag(pocsag_message)) => Some(pocsag_message),
            _ => None
        };

        match self.message
        {
            Some(_) => State::AddressWord,
            None => State::Completed,
        }
    }
}

// Calculate the CRC for a codeword and return the updated codeword.
fn crc(codeword: u32) -> u32 {
    let mut crc = codeword;
    for i in 0..=21 {
        if (crc & (0x80000000 >> i)) != 0 {
            crc ^= 0xED200000 >> i;
        }
    }
    codeword | crc
}

// Calculate the parity bit for a codeword and return the updated codeword.
fn parity(codeword: u32) -> u32 {
    let mut parity = codeword ^ (codeword >> 1);
    parity ^= parity >> 2;
    parity ^= parity >> 4;
    parity ^= parity >> 8;
    parity ^= parity >> 16;
    codeword | (parity & 1)
}

impl<'a> Iterator for Generator<'a> {
    // The Iterator returns 32-bit codewords.
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        trace!("Next generated codeword: ({}, {:?})", self.codewords, self.state);
        self.count += 1;

        match (self.codewords, self.state)
        {
            // Terminator batch finished: end the iterator.
            (0, State::Terminator) => None,

            // End of the last batch with a pending message-end: emit the
            // trailing sync word and enter the terminator batch.
            (0, State::Completed) => {
                self.codewords = 16;
                self.state = State::Terminator;
                Some(SYNC_WORD)
            }

            // The preamble is completed.
            // Send the sync word and start a new batch with 16 codewords.
            (0, State::Preamble) => {
                self.codewords = 16;
                self.state = State::AddressWord;
                Some(SYNC_WORD)
            }

            // No codewords left in the current batch.
            // Send the sync word and start a new batch with 16 codewords.
            (0, _) => {
                self.codewords = 16;
                Some(SYNC_WORD)
            }

            // There are still preamble codewords left to send.
            (_, State::Preamble) => {
                self.codewords -= 1;
                Some(0xAAAAAAAA)
            }

            // Send the address word for the current message
            (codeword, State::AddressWord) => {
                let length =
                    self.message.as_ref().map(|m| m.data.len()).unwrap_or(0);

                let &Message { ric, func, mtype, .. } =
                    self.message.as_ref().unwrap();

                self.codewords -= 1;

                // Send idle words until the current batch position
                // matches the position required by the subric.
                if ((ric & 0b111) << 1) as u8 == 16 - codeword {
                    // Set the next state according to the message type
                    self.state = if length == 0 {
                        self.next_message()
                    }
                    else {
                        match mtype
                        {
                            MessageType::Numeric => {
                                State::MessageWord(0, encoding::NUMERIC)
                            }
                            MessageType::AlphaNum => {
                                State::MessageWord(0, encoding::ALPHANUM)
                            }
                        }
                    };

                    // Encode the address word.
                    let addr = (ric & 0x001ffff8) << 10;
                    let func = (func as u32 & 0b11) << 11;
                    Some(parity(crc(addr | func)))
                }
                else {
                    Some(IDLE_WORD)
                }
            }

            // Send the next message word of the current message.
            (_, State::MessageWord(pos, encoding)) => {
                self.codewords -= 1;
                let mut pos = pos;
                let mut codeword: u32 = 0;

                let completed = {
                    let message = self.message.as_ref().unwrap();
                    let mut bytes = message.data.bytes();

                    // Get the next symbol and shift it to start with correct
                    // bit.
                    let mut sym = bytes
                        .nth(pos / encoding.bits)
                        .map(encoding.encode)
                        .unwrap_or(encoding.trailing) >>
                        (pos % encoding.bits);

                    for _ in 0..20 {
                        // Add the next bit of the symbol to the codeword.
                        codeword = (codeword << 1) | (sym & 1) as u32;

                        pos += 1;

                        // If all bits are send, continue with the next symbol.
                        if pos % encoding.bits == 0 {
                            sym = bytes.next().map(encoding.encode).unwrap_or(
                                encoding.trailing
                            );
                        }
                        else {
                            sym >>= 1;
                        }
                    }

                    // If no symbols are left, the message is completed.
                    pos > message.data.len() * encoding.bits
                };

                // Continue with the next message if the current one is
                // completed.
                self.state = if completed {
                    self.next_message()
                }
                else {
                    State::MessageWord(pos, encoding)
                };

                // TODO: ensure that an trailing IDLE, SYNC or ADDR word is sent

                Some(parity(crc(0x80000000 | (codeword << 11))))
            }

            // No more messages. Fill the rest of the current batch (Completed)
            // or the terminator batch (Terminator) with idle words.
            (_, State::Completed) | (_, State::Terminator) => {
                self.codewords -= 1;
                Some(IDLE_WORD)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Message as OuterMessage, MessageProvider};
    use crate::pocsag::{Message, MessageType};

    struct EmptyProvider;

    impl MessageProvider for EmptyProvider {
        fn next(&mut self, _count: usize) -> Option<OuterMessage> {
            None
        }
    }

    fn collect_all(ric: u32, length: usize) -> Vec<u32> {
        let data: String = std::iter::repeat('A').take(length).collect();
        let msg = Message {
            mtype: MessageType::AlphaNum,
            speed: 1200,
            ric,
            func: 3,
            data,
        };
        let mut provider = EmptyProvider;
        Generator::new(&mut provider, msg).collect()
    }

    #[test]
    fn terminator_follows_last_message_word() {
        for length in 1..=60usize {
            for ric_lower in 0..=7u32 {
                // Build a RIC where the lower three bits vary; the upper bits
                // are arbitrary but valid.
                let ric = 0x0010_0000 | ric_lower;
                let cws = collect_all(ric, length);

                assert!(!cws.is_empty(),
                    "no codewords produced (length={}, ric_lower={})",
                    length, ric_lower);

                // The final codeword in the stream must not be a message word.
                let last = *cws.last().unwrap();
                assert_eq!(last & 0x80000000, 0,
                    "last codeword is a message word \
                     (length={}, ric_lower={}, last=0x{:08X})",
                    length, ric_lower, last);

                // At least one IDLE_WORD must appear between the last message
                // word in the stream and the end.
                let last_msg_idx = cws.iter()
                    .rposition(|&w| w & 0x80000000 != 0)
                    .expect("stream must contain at least one message word");
                let after = &cws[last_msg_idx + 1..];
                assert!(after.iter().any(|&w| w == IDLE_WORD),
                    "no idle word follows the last message word \
                     (length={}, ric_lower={}, tail_len={})",
                    length, ric_lower, after.len());
            }
        }
    }
}
