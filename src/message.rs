use std::str::FromStr;

#[derive(Debug)] pub enum MessageSpeed { Baud512, Baud1200, Baud2400 }
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

}

impl FromStr for MessageSpeed {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageSpeed, Self::Err> {
        match s {
            "0" => Ok(MessageSpeed::Baud512),
            "1" => Ok(MessageSpeed::Baud1200),
            "2" => Ok(MessageSpeed::Baud2400),
            _ => Err(())
        }
    }
}

impl FromStr for MessageType {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageType, Self::Err> {
        match s {
            "2" => Ok(MessageType::SyncRequest),
            "3" => Ok(MessageType::SyncOrder),
            "4" => Ok(MessageType::Slots),
            "5" => Ok(MessageType::Numeric),
            "6" => Ok(MessageType::AlphaNum),
            _ => Err(())
        }
    }
}

impl FromStr for MessageFunc {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageFunc, Self::Err> {
        match s {
            "0" => Ok(MessageFunc::Numeric),
            "1" => Ok(MessageFunc::Tone),
            "2" => Ok(MessageFunc::Activation),
            "3" => Ok(MessageFunc::AlphaNum),
            _ => Err(())
        }
    }
}
