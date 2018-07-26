pub mod http;
pub mod websocket;

use serde_json;

use config::Config;
use message::Message;
use telemetry::Telemetry;

#[derive(Debug, Deserialize)]
pub enum Request {
    SetConfig(Config),
    DefaultConfig,
    SendMessage(Message),
    Authenticate(String),
    GetConfig,
    GetTelemetry,
    GetVersion,
    Test
}

#[derive(Debug, Serialize)]
pub enum Response {
    Config(Config),
    Telemetry(Telemetry),
    TelemetryUpdate(serde_json::Value),
    Version(String),
    Message(Message),
    Log(u8, String),
    Authenticated(bool)
}
