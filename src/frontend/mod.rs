pub mod http;
pub mod websocket;

use serde_json;

use config::Config;
use status::Status;
use pocsag::Message;

#[derive(Debug, Deserialize)]
pub enum Request {
    SetConfig(Config),
    DefaultConfig,
    SendMessage(Message),
    Authenticate(String),
    GetConfig,
    GetVersion,
    GetStatus,
    Shutdown,
    Restart,
    Test
}

#[derive(Debug, Serialize)]
pub enum Response {
    Status(Status),
    StatusUpdate(String, serde_json::value::Value),
    Config(Config),
    Version(String),
    Message(Message),
    Log(u8, String),
    Authenticated(bool)
}
