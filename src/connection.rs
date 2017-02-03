use std::net::{TcpStream, Shutdown};
use std::io::{BufReader, BufWriter, BufRead, Write, Result};
use std::time::Duration;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{Sender, channel};
use std::str::{FromStr};

use config::Config;
use pocsag::{Scheduler, TimeSlots};
use pocsag::{Message, MessageSpeed, MessageType, MessageFunc};

pub struct Connection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
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
    pub fn new(config: &Config, scheduler: Scheduler) -> Result<Connection> {
        let addr = (&*config.master.server, config.master.port);
        let stream = TcpStream::connect(addr)?;
        stream.set_write_timeout(Some(Duration::from_millis(10000)))?;

        Ok(Connection {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream.try_clone()?),
            stream: stream,
            scheduler: scheduler,
            auth: config.master.auth.to_owned(),
            id: config.transmitter.to_string()
        })
    }

    pub fn start(config: Config, scheduler: Scheduler) -> (Sender<()>, JoinHandle<()>) {
        let (stop_tx, stop_rx) = channel();
        let mut reconnect = true;
        let mut delay = Duration::from_millis(1000);

        let handle = thread::spawn(move || {
            while reconnect {
                info!("Trying to connect to master...");
                let connection = Connection::new(&config, scheduler.clone());

                if let Ok(mut connection) = connection {
                    info!("Connection established.");
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

                    stream.shutdown(Shutdown::Both).unwrap();
                    handle.join().unwrap();

                    info!("Connection closed.");
                    delay = Duration::from_millis(1000);
                }
                else {
                    error!("Connection failed.");
                    delay = Duration::from_millis(5000);
                }
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
        let id = format!("[{} v1.0 {}]\r\n", self.id, self.auth);
        self.writer.write(id.as_bytes())?;
        self.writer.flush()?;

        let mut buffer = String::new();

        while self.reader.read_line(&mut buffer)? > 0 {
            self.handle(&*buffer)?;
            buffer.clear();
        }
        Ok(())
    }

    fn send(&mut self, data: &str) -> Result<()> {
        debug!("Response: {}", data);
        self.writer.write(data.as_bytes())?;
        self.writer.write(b"\r\n")?;
        self.writer.flush()?;
        Ok(())
    }

    fn ack(&mut self, status: AckStatus) -> Result<()> {
        let response = match status {
            AckStatus::Success => b"+\r\n",
            AckStatus::Error => b"-\r\n",
            AckStatus::Retry => b"%\r\n",
            AckStatus::Nothing => return Ok(())
        };
        self.writer.write(response)?;
        self.writer.flush()?;
        Ok(())
    }

    fn handle(&mut self, data: &str) -> Result<()> {
        let data = data.trim();
        debug!("Received data: {}", data);

        if data.len() < 1 { return Ok(()); }

        let status = match &data[0..1] {
            "#" => self.handle_message(data),
            "2" => self.handle_ident(data),
            "3" => Ok(AckStatus::Success),
            "4" => self.handle_timeslots(data),
            other => {
                error!("Unknown message received from server (type: {})", other);
                Ok(AckStatus::Error)
            }
        };

        if let Ok(status) = status {
            self.ack(status)?;
        }
        Ok(())
    }

    fn handle_message(&mut self, data: &str) -> Result<AckStatus> {
        let mut parts = data.split(":").peekable();

        let msg_id = parts.peek().and_then(|str| u8::from_str_radix(&str[1..3], 16).ok());
        let msg_type = parts.next().and_then(|str| MessageType::from_str(&str[4..5]).ok());
        let msg_speed = parts.next().and_then(|str| MessageSpeed::from_str(&str).ok());
        let msg_addr = parts.next().and_then(|str| u32::from_str_radix(&str, 16).ok());
        let msg_func = parts.next().and_then(|str| MessageFunc::from_str(&str).ok());
        let msg_data: String = parts.collect::<Vec<&str>>().join(":");

        if msg_id.is_some() && msg_type.is_some() && msg_addr.is_some() && msg_func.is_some() {
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
        }
        else {
            error!("Malformed message received: {}", data);
            Ok(AckStatus::Error)
        }
    }

    fn handle_ident(&mut self, data: &str) -> Result<AckStatus> {
        let ident = data.split(":").nth(1).unwrap_or("");
        self.send(&*format!("2:{}:{:04x}", ident, 0))?;
        Ok(AckStatus::Success)
    }

    fn handle_timeslots(&mut self, data: &str) -> Result<AckStatus> {
        if let Some(slots) = data.split(":").nth(1) {
            let time_slots = TimeSlots::from_str(slots).unwrap();
            self.scheduler.set_time_slots(time_slots);
            Ok(AckStatus::Success)
        }
        else {
            Ok(AckStatus::Error)
        }
    }
}
