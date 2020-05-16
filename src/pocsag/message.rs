#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Numeric,
    AlphaNum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Message {
    #[serde(rename = "type")]
    pub mtype: MessageType,
    pub speed: u32,
    pub ric: u32,
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
            mtype: MessageType::AlphaNum,
            speed: 1200,
            ric: 0,
            func: 3,
            data: "".to_owned()
        }
    }
}
