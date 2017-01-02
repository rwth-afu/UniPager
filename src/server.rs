use std::net::TcpListener;
use std::net::IpAddr;
use std::str::FromStr;
use std::thread;

use config::Config;
use connection::Connection;
use pocsag::scheduler::Scheduler;

pub struct Server {
    listener: TcpListener,
    master: Option<IpAddr>
}

impl Server {
    pub fn new(config: &Config) -> Server {
        let port = config.master.port as u16;
        let master = IpAddr::from_str(&*config.master.server).ok();

        if let Some(addr) = master {
            info!("Listening for {} on port {}.", addr, port);
        }
        else {
            info!("Listening for ALL HOSTS on port {}.", port);
        }

        let listener = TcpListener::bind(("0.0.0.0", port)).expect("Unable to listen");

        Server {
            listener: listener,
            master: master
        }
    }

    pub fn run(&self, scheduler: Scheduler) {
        for stream in self.listener.incoming() {
            if let Ok(stream) = stream {
                let addr = stream.peer_addr().ok().map(|addr| addr.ip());

                if self.master.is_none() || addr == self.master {
                    info!("Client connected: {}", addr.unwrap());
                    let mut connection = Connection::new(stream, scheduler.clone());
                    thread::spawn(move || connection.run());
                }
                else {
                    info!("Client denied: {}", addr.unwrap());
                }
            }
        }
    }
}
