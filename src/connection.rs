use std::io::{self, BufRead, BufReader, BufWriter, Result, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::sync::mpsc::{Sender, channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration};

use config::Config;
use pocsag::{Message, MessageFunc, MessageSpeed, MessageType};
use pocsag::{Scheduler, TimeSlots};

pub struct Connection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
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
            info!("Connection to {}:{}...", host, port);

            let addr = (&**host, port).to_socket_addrs()?
                .next().ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Cannot resolve hostname"
                ))?;

            let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(10000))?;
            stream.set_write_timeout(Some(Duration::from_millis(10000)))?;
            stream.set_read_timeout(Some(Duration::from_millis(125000)))?;

            Ok(Connection {
                reader: BufReader::new(stream.try_clone()?),
                writer: BufWriter::new(stream.try_clone()?),
                stream: stream,
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

                    let stream = connection.stream.try_clone().unwrap();

                    let (stopped_tx, stopped_rx) = channel();
                    let handle = thread::spawn(move || {
                        connection.run().ok();
                        stopped_tx.send(()).unwrap();
                    });

                    select! {
                        _ = stopped_rx.recv() => reconnect = true,
                        _ = stop_rx.recv() => reconnect = false
                    }

                    stream.shutdown(Shutdown::Both).ok();
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
        let version = env!("CARGO_PKG_VERSION");
        let id = format!(
            "[UniPager-{} v{} {} {}]",
            self.id,
            version,
            self.call,
            self.auth
        );

        self.send(&*id)?;

        let mut buffer = String::new();

        while self.reader.read_line(&mut buffer)? > 0 {
            self.handle(&*buffer)?;
            buffer.clear();
        }

        Ok(())
    }

    fn send(&mut self, data: &str) -> Result<()> {
        debug!("Response: {}", data);
        self.writer.write_all(data.as_bytes())?;
        self.writer.write_all(b"\r\n")?;
        self.writer.flush()?;
        Ok(())
    }

    fn ack(&mut self, status: AckStatus) -> Result<()> {
        let response = match status {
            AckStatus::Success => b"+\r\n",
            AckStatus::Error => b"-\r\n",
            AckStatus::Retry => b"%\r\n",
            AckStatus::Nothing => return Ok(()),
        };
        self.writer.write_all(response)?;
        self.writer.flush()?;
        Ok(())
    }

    fn handle(&mut self, data: &str) -> Result<()> {
        let data = data.trim();
        debug!("Received data: {}", data);

        if data.len() < 1 {
            return Ok(());
        }

        let status = match &data[0..1] {
            "#" => self.handle_message(data),
            "2" => self.handle_ident(data),
            "3" => Ok(AckStatus::Success),
            "4" => self.handle_timeslots(data),
            "7" => self.handle_failedlogin(data),
            other => {
                error!(
                    "Unknown message received from server (type: {})",
                    other
                );
                Ok(AckStatus::Error)
            }
        };

        if let Ok(status) = status {
            self.ack(status)?;
        }
        Ok(())
    }

    fn handle_message(&mut self, data: &str) -> Result<AckStatus> {
        let mut parts = data.split(':').peekable();

        let msg_id = parts.peek().and_then(
            |s| u8::from_str_radix(&s[1..3], 16).ok()
        );
        let msg_type = parts.next().and_then(
            |s| MessageType::from_str(&s[4..5]).ok()
        );
        let msg_speed =
            parts.next().and_then(|s| MessageSpeed::from_str(s).ok());
        let msg_addr =
            parts.next().and_then(|s| u32::from_str_radix(s, 16).ok());
        let msg_func = parts.next().and_then(|s| MessageFunc::from_str(s).ok());
        let msg_data: String = parts.collect::<Vec<&str>>().join(":");

        if msg_id.is_some() && msg_type.is_some() && msg_addr.is_some() &&
            msg_func.is_some()
        {
            let msg = Message {
                id: msg_id.unwrap(),
                mtype: msg_type.unwrap(),
                speed: msg_speed.unwrap_or(MessageSpeed::Baud(1200)),
                addr: msg_addr.unwrap(),
                func: msg_func.unwrap(),
                data: msg_data
            };

            let next_id = (msg.id as u16 + 1) % 256;
            self.scheduler.message(msg);
            self.send(&*format!("#{:02x} +", next_id))?;
            Ok(AckStatus::Nothing)
        } else {
            error!("Malformed message received: {}", data);
            Ok(AckStatus::Error)
        }
    }

    fn handle_ident(&mut self, data: &str) -> Result<AckStatus> {
        let ident = data.split(':').nth(1).unwrap_or("");
        self.send(&*format!("2:{}:{:04x}", ident, 0))?;
        Ok(AckStatus::Success)
    }

    fn handle_timeslots(&mut self, data: &str) -> Result<AckStatus> {
        if let Some(slots) = data.split(':').nth(1) {
            let time_slots = TimeSlots::from_str(slots).unwrap();
            self.scheduler.set_time_slots(time_slots);
            Ok(AckStatus::Success)
        } else {
            Ok(AckStatus::Error)
        }
    }

    fn handle_failedlogin(&mut self, data: &str) -> Result<AckStatus> {
        error!("Transmitter login failed. Reason: {}", &data[2..]);
        Ok(AckStatus::Error)
    }
}
