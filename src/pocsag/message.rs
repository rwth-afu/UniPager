use std::str::FromStr;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MessageType {
    Numeric,
    AlphaNum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Message {
    pub id: String,
    pub mtype: MessageType,
    pub speed: u32,
    pub addr: u32,
    pub func: u8,
    pub data: String
}

impl Message {
    pub fn size(&self) -> usize {
        // TODO: Calculate worst case size
        self.data.len()
    }
}

impl Default for Message {
    fn default() -> Message {
        Message {
            id: "".to_owned(),
            mtype: MessageType::AlphaNum,
            speed: 1200,
            addr: 0,
            func: 3,
            data: "".to_owned()
        }
    }
}

impl FromStr for MessageType {
    type Err = ();

    fn from_str(s: &str) -> Result<MessageType, Self::Err> {
        match u8::from_str(s) {
            Ok(5) => Ok(MessageType::Numeric),
            Ok(6) => Ok(MessageType::AlphaNum),
            _ => Err(()),
        }
    }
}
