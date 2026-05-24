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
    Completed
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
    for i in 0..21 {
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
            // Stop if no codewords are left and everything is completed.
            (0, State::Completed) => None,

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

            // Everything is done. Send idle words until the batch is complete.
            (_, State::Completed) => {
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

    // ---- BCH(31,21) verification helpers ----

    // Re-divide a 32-bit POCSAG codeword by the BCH(31,21) generator
    // polynomial g(x) = x^10 + x^9 + x^8 + x^6 + x^5 + x^3 + 1.
    // A correctly encoded codeword has remainder 0.
    fn bch_syndrome(codeword: u32) -> u32 {
        let mut c = codeword & !1u32; // exclude the parity bit at position 0
        for i in 0..21 {
            if (c & (0x80000000u32 >> i)) != 0 {
                c ^= 0xED200000u32 >> i;
            }
        }
        (c >> 1) & 0x3FF
    }

    fn even_parity_bit(codeword: u32) -> u32 {
        let mut p = codeword;
        p ^= p >> 1; p ^= p >> 2; p ^= p >> 4; p ^= p >> 8; p ^= p >> 16;
        p & 1
    }

    // Independent BCH encoder in the multimon-ng / libbch_pocsag "left-shift"
    // style. Used to cross-check our right-shift implementation.
    fn bch_encode_reference(cw: u32) -> u32 {
        let masked = cw & 0xFFFFF800; // clear BCH+parity region
        let mut local = masked;
        for _ in 0..21 {
            if local & 0x80000000 != 0 {
                local ^= 0xED200000;
            }
            local <<= 1;
        }
        let mut with_bch = masked | (local >> 21);
        if even_parity_bit(with_bch) == 1 {
            with_bch |= 1;
        }
        with_bch
    }

    // ---- Regression vectors ----

    // The well-known POCSAG IDLE codeword must round-trip:
    // strip BCH+parity, re-encode, and the result must equal IDLE_WORD.
    #[test]
    fn idle_word_roundtrips() {
        let raw = IDLE_WORD & 0xFFFFF800;
        let encoded = parity(crc(raw));
        assert_eq!(encoded, IDLE_WORD,
            "IDLE_WORD round-trip: got 0x{:08X}, expected 0x{:08X}",
            encoded, IDLE_WORD);
    }

    // Explicit input/output pairs.  Each expected value is also verified to
    // be a true BCH(31,21) codeword (syndrome zero) with even parity, so the
    // assertion catches both encoder bugs and stale test vectors.
    //
    //   data=0x000001 -> only bit 0 of the 21-bit data set (codeword bit 11)
    //     x^10 mod g = x^9+x^8+x^6+x^5+x^3+1 -> BCH at bits 10..1 = 0x6D2
    //     parity over 0x800|0x6D2 is odd, so bit 0=1 -> 0xED3
    //   data=0x100000 -> only bit 20 of data set (codeword bit 31, message
    //     identifier).  x^30 mod g = 0x3B4 -> BCH at bits 10..1 = 0x768
    //     parity is odd, bit 0=1 -> 0x80000769
    #[test]
    fn known_codewords() {
        let cases: &[(u32, u32)] = &[
            (0x000000, 0x00000000),
            (0x000001, 0x00000ED3),
            (0x100000, 0x80000769),
        ];
        for &(data, expected) in cases {
            let raw = data << 11;
            let got = parity(crc(raw));
            assert_eq!(got, expected,
                "data=0x{:06X}: got 0x{:08X}, expected 0x{:08X}",
                data, got, expected);
            // Sanity-check that the expected value really is a valid codeword.
            assert_eq!(bch_syndrome(expected), 0);
            assert_eq!(even_parity_bit(expected), 0);
        }
    }

    // The original bug (22-iteration loop) produced wrong BCH bits whenever
    // bit 10 of the correct check was 1.  Exhaustively verify our crc() over
    // every 21-bit input, comparing against the independent left-shift
    // reference encoder.
    #[test]
    fn crc_matches_reference_encoder_for_all_inputs() {
        for data in 0u32..(1u32 << 21) {
            let raw = data << 11;
            let ours = parity(crc(raw));
            let reference = bch_encode_reference(raw);
            assert_eq!(ours, reference,
                "mismatch at data=0x{:06X}: ours=0x{:08X}, ref=0x{:08X}",
                data, ours, reference);
        }
    }

    // Every codeword our crc()+parity() produces must satisfy the BCH(31,21)
    // syndrome check and have even parity over all 32 bits.
    #[test]
    fn all_codewords_have_valid_bch_and_parity() {
        for data in 0u32..(1u32 << 21) {
            let raw = data << 11;
            let cw = parity(crc(raw));
            assert_eq!(bch_syndrome(cw), 0,
                "BCH syndrome nonzero for data=0x{:06X}, cw=0x{:08X}",
                data, cw);
            assert_eq!(even_parity_bit(cw), 0,
                "parity not even for data=0x{:06X}, cw=0x{:08X}",
                data, cw);
        }
    }

    // ---- End-to-end Generator regression ----
    //
    // Drive the full Generator and check that every emitted address word and
    // message word is a valid BCH(31,21) codeword. Sweeps message lengths and
    // sub-RIC values so the codeword data field varies widely.

    struct EmptyProvider;
    impl MessageProvider for EmptyProvider {
        fn next(&mut self, _count: usize) -> Option<OuterMessage> {
            None
        }
    }

    fn generate_codewords(ric: u32, length: usize) -> Vec<u32> {
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
    fn generator_emits_valid_bch_codewords() {
        // Skip preamble (0xAAAAAAAA) and sync words; everything else must be
        // a valid BCH+parity codeword.
        for length in 0..=30usize {
            for ric_lower in 0..=7u32 {
                let ric = 0x0010_0000 | ric_lower;
                for (idx, &cw) in generate_codewords(ric, length)
                    .iter().enumerate()
                {
                    if cw == SYNC_WORD || cw == 0xAAAAAAAA {
                        continue;
                    }
                    assert_eq!(bch_syndrome(cw), 0,
                        "BCH syndrome nonzero at idx {} (length={}, \
                         ric_lower={}, cw=0x{:08X})",
                        idx, length, ric_lower, cw);
                    assert_eq!(even_parity_bit(cw), 0,
                        "parity not even at idx {} (length={}, \
                         ric_lower={}, cw=0x{:08X})",
                        idx, length, ric_lower, cw);
                }
            }
        }
    }
}
