pub mod http;
pub mod websocket;

pub use self::websocket::Responder;

use std::sync::mpsc::{Receiver, channel};
use std::thread;

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
    Config(Config),
    Version(String),
    Message(Message),
    Log(u8, String),
    Authenticated(bool)
}

pub fn run(pass: Option<&str>) -> (Responder, Receiver<Request>) {
    thread::spawn(http::run);

    let (tx, rx) = channel();
    let responder = websocket::create(tx, pass);

    (responder, rx)
}
