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
    Log(u8, String)
}

pub fn run() -> (Responder, Receiver<Request>) {
    thread::spawn(http::run);

    let (tx, rx) = channel();
    let responder = websocket::create(tx);

    (responder, rx)
}
