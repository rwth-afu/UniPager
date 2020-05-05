pub mod http;
pub mod websocket;

use serde_json;

use crate::config::Config;
use crate::message::Message;
use crate::telemetry::Telemetry;
use crate::timeslots::TimeSlot;

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
    Restart,
    Shutdown,
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
