use std::io::{self, BufRead, BufReader, BufWriter, Result, Write};
use std::str::FromStr;
use std::sync::mpsc::{Sender, channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration};
use amqp::{Basic, Session, Channel, Table, protocol};

use config::Config;
use pocsag::{Message, MessageFunc, MessageSpeed, MessageType};
use pocsag::{Scheduler, TimeSlots};

pub struct Connection {
    session: Session,
    channel: Channel,
    call: String,
    auth: String,
    id: String,
    scheduler: Scheduler
}

#[allow(dead_code)]
enum AckStatus {
    Success,
    Error,
    Retry,
    Nothing
}

impl Connection {
    pub fn new(host: &String, port: u16, config: &Config, scheduler: Scheduler)
               -> Result<Connection> {
        if config.master.call.len() == 0 {
            error!("No callsign configured.");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No callsign configured"
            ))
        } else if config.master.auth.len() == 0 {
            error!("No auth key configured.");
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No auth key configured"
            ))
        } else {
            info!("Connecting to {}:{}...", host, port);

            let addr = format!("amqp://guest:guest@{}:{}", host, port);

            let mut session = Session::open_url(&addr).map_err(|err|
                                                                  io::Error::new(
                                                                      io::ErrorKind::InvalidInput,
                                                                      "No auth key configured"
                                                                  ))?;
            let mut channel = session.open_channel(1).map_err(|err|
                                                              io::Error::new(
                                                                  io::ErrorKind::InvalidInput,
                                                                  "No auth key configured"
                                                              ))?;

            Ok(Connection {
                session: session,
                channel: channel,
                scheduler: scheduler,
                call: config.master.call.to_owned(),
                auth: config.master.auth.to_owned(),
                id: config.transmitter.to_string()
            })
        }
    }

    pub fn start(config: Config, scheduler: Scheduler)
        -> (Sender<()>, JoinHandle<()>) {
        let (stop_tx, stop_rx) = channel();

        let handle = thread::spawn(move || {
            let mut reconnect = true;
            let mut try_fallback = false;
            let mut fallback = config.master.fallback.iter().cycle();

            while reconnect {
                let (ref host, port) = if try_fallback {
                    warn!("Connecting to next fallback server...");
                    if let Some(&(ref host, port)) = fallback.next() {
                        (host, port)
                    }
                    else {
                        error!("No fallback servers defined.");
                        (&config.master.server, config.master.port)
                    }
                }
                else {
                    (&config.master.server, config.master.port)
                };

                let connection = Connection::new(&host, port, &config, scheduler.clone());

                let delay = if let Ok(mut connection) = connection {
                    info!("Connection established.");
                    status!(connected: true, master: Some(host.to_string()));
                    try_fallback = false;

                    let (stopped_tx, stopped_rx) = channel();
                    let handle = thread::spawn(move || {
                        connection.run().ok();
                        stopped_tx.send(()).unwrap();
                    });

                    select! {
                        _ = stopped_rx.recv() => reconnect = true,
                        _ = stop_rx.recv() => reconnect = false
                    }

                    handle.join().unwrap();

                    status!(connected: false, master: None::<String>);
                    warn!("Disconnected from master.");

                    Duration::from_millis(2500)
                } else {
                    status!(connected: false, master: None::<String>);
                    error!("Connection failed.");

                    try_fallback = !try_fallback;

                    Duration::from_millis(5000)
                };

                if reconnect {
                    if let Ok(()) = stop_rx.recv_timeout(delay) {
                        reconnect = false;
                    }
                }
            }
        });

        (stop_tx, handle)
    }

    pub fn run(&mut self) -> Result<()> {

        Ok(())
    }
}
