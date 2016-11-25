use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)] pub enum MessageSpeed { Baud(usize) }
#[derive(Debug)] pub enum MessageType { SyncRequest, SyncOrder, Slots, Numeric, AlphaNum }
#[derive(Debug)] pub enum MessageFunc { Numeric, Tone, Activation, AlphaNum }

#[derive(Debug)]
pub struct Message {
    pub id: u8,
    pub mtype: MessageType,
    pub speed: MessageSpeed,
    pub addr: u32,
    pub func: MessageFunc,
    pub text: String
}

impl Message {
    // pub fn duration() -> Duration {
    //     // Frame Sync (32 bit)
    //     let mut bits = 32 + 8*16;
    // }
}

impl FromStr for MessageSpeed {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageSpeed, Self::Err> {
        match u8::from_str(&s) {
            Ok(0) => Ok(MessageSpeed::Baud(512)),
            Ok(1) => Ok(MessageSpeed::Baud(1200)),
            Ok(2) => Ok(MessageSpeed::Baud(2400)),
            _ => Err(())
        }
    }
}

impl FromStr for MessageType {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageType, Self::Err> {
        match u8::from_str(&s) {
            Ok(2) => Ok(MessageType::SyncRequest),
            Ok(3) => Ok(MessageType::SyncOrder),
            Ok(4) => Ok(MessageType::Slots),
            Ok(5) => Ok(MessageType::Numeric),
            Ok(6) => Ok(MessageType::AlphaNum),
            _ => Err(())
        }
    }
}

impl FromStr for MessageFunc {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageFunc, Self::Err> {
        match u8::from_str(&s) {
            Ok(0) => Ok(MessageFunc::Numeric),
            Ok(1) => Ok(MessageFunc::Tone),
            Ok(2) => Ok(MessageFunc::Activation),
            Ok(3) => Ok(MessageFunc::AlphaNum),
            _ => Err(())
        }
    }
}
