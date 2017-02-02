use std::net::TcpStream;
use std::io::{BufReader, BufWriter, BufRead, Write, Result};
use std::time::Duration;
use std::str::{FromStr};

use config::Config;
use pocsag::{Scheduler, TimeSlots};
use pocsag::{Message, MessageSpeed, MessageType, MessageFunc};

pub struct Connection {
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
        stream.set_read_timeout(Some(Duration::from_millis(30000)))?;
        stream.set_write_timeout(Some(Duration::from_millis(30000)))?;

        Ok(Connection {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
            scheduler: scheduler,
            auth: config.master.auth.to_owned(),
            id: config.transmitter.to_string()
        })
    }

    pub fn run(&mut self) {
        let id = format!("[{} v1.0 {}]\r\n", self.id, self.auth);
        self.writer.write(id.as_bytes()).unwrap();
        self.writer.flush().unwrap();

        let mut buffer = String::new();

        while self.reader.read_line(&mut buffer).unwrap() > 0 {
            self.handle(&*buffer);
            buffer.clear();
        }
    }

    fn send(&mut self, data: &str) {
        debug!("Response: {}", data);
        self.writer.write(data.as_bytes()).unwrap();
        self.writer.write(b"\r\n").unwrap();
        self.writer.flush().unwrap();
    }

    fn ack(&mut self, status: AckStatus) {
        let response = match status {
            AckStatus::Success => b"+\r\n",
            AckStatus::Error => b"-\r\n",
            AckStatus::Retry => b"%\r\n",
            AckStatus::Nothing => return
        };
        self.writer.write(response).unwrap();
        self.writer.flush().unwrap();
    }

    fn handle(&mut self, data: &str) {
        let data = data.trim();
        debug!("Received data: {}", data);

        if data.len() < 1 { return; }

        let status = match &data[0..1] {
            "#" => self.handle_message(data),
            "2" => self.handle_ident(data),
            "3" => AckStatus::Success,
            "4" => self.handle_timeslots(data),
            other => {
                error!("Unknown message received from server (type: {})", other);
                AckStatus::Error
            }
        };

        self.ack(status);
    }

    fn handle_message(&mut self, data: &str) -> AckStatus {
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
            self.send(&*format!("#{:02x} +", next_id));
            AckStatus::Nothing
        }
        else {
            error!("Malformed message received: {}", data);
            AckStatus::Error
        }
    }

    fn handle_ident(&mut self, data: &str) -> AckStatus {
        let ident = data.split(":").nth(1).unwrap_or("");
        self.send(&*format!("2:{}:{:04x}", ident, 0));
        AckStatus::Success
    }

    fn handle_timeslots(&mut self, data: &str) -> AckStatus {
        if let Some(slots) = data.split(":").nth(1) {
            let time_slots = TimeSlots::from_str(slots).unwrap();
            self.scheduler.set_time_slots(time_slots);
            AckStatus::Success
        }
        else {
            AckStatus::Error
        }
    }
}
