pub mod http;
pub mod websocket;

use serde_json;

use config::Config;
use message::Message;
use telemetry::Telemetry;
use timeslots::TimeSlot;

#[derive(Debug, Deserialize)]
pub enum Request {
    SetConfig(Config),
    DefaultConfig,
    SendMessage(Message),
    Authenticate(String),
    GetConfig,
    GetTelemetry,
    GetTimeslot,
    GetVersion,
    Test
}

#[derive(Debug, Serialize)]
pub enum Response {
    Config(Config),
    Telemetry(Telemetry),
    TelemetryUpdate(serde_json::Value),
    Timeslot(TimeSlot),
    Version(String),
    Message(Message),
    Log(u8, String),
    Authenticated(bool)
}
