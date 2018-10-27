use chrono::prelude::*;

use pocsag;

pub trait MessageProvider {
    fn next(&mut self, count: usize) -> Option<Message>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol", content = "message")]
#[serde(rename_all = "lowercase")]
pub enum ProtocolMessage {
    Pocsag(pocsag::Message)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub priority: usize,
    pub origin: String,
    #[serde(default)]
    pub expires_on: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub message: ProtocolMessage
}

impl Message {
    pub fn is_expired(&self) -> bool {
        match self.expires_on
        {
            Some(time) => Utc::now() >= time,
            _ => false,
        }
    }

    pub fn generator<'a>(self, provider: &'a mut MessageProvider)
        -> Box<Iterator<Item = u32> + 'a> {
        match self.message
        {
            ProtocolMessage::Pocsag(msg) => {
                Box::new(pocsag::Generator::new(provider, msg))
            }
        }
    }
}
