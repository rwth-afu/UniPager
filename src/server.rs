use std::net::TcpListener;
use std::thread;

use connection::Connection;

pub struct Server {
    listener: TcpListener
}

impl Server {
    pub fn new() -> Server {
        let listener = TcpListener::bind("0.0.0.0:1337").expect("Unable to listen");
        Server {
            listener: listener
        }
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            if let Ok(stream) = stream {
                info!("Client connected: {}", stream.peer_addr().unwrap());
                let mut connection = Connection::new(stream);
                thread::spawn(move || connection.run());
            }
        }
    }
}
