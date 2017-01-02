pub mod http;
pub mod websocket;

use std::thread;

pub fn run() {
    thread::spawn(http::run);
    websocket::run();
}
