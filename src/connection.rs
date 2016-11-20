use std::net::TcpStream;
use std::io::{BufReader, BufWriter, BufRead, Write};
use std::str;
use std::str::{FromStr};

use scheduler::Scheduler;
use message::{Message, MessageSpeed, MessageType, MessageFunc};

pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    scheduler: Scheduler
}

enum AckStatus {
    Success,
    Error,
    Retry
}

impl Connection {
    pub fn new(stream: TcpStream, scheduler: Scheduler) -> Connection {
        let stream1 = stream.try_clone().unwrap();
        let stream2 = stream1.try_clone().unwrap();

        Connection {
            reader: BufReader::new(stream1),
            writer: BufWriter::new(stream2),
            scheduler: scheduler
        }
    }

    pub fn run(&mut self) {
        self.writer.write(b"[SDRPager v1.2-SCP-#2345678]\r\n").unwrap();
        self.writer.flush().unwrap();

        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).unwrap();
            self.handle(&*line);
        }
    }

    fn handle(&mut self, data: &str) {
        debug!("Received data: {}", data.trim());
        let mut parts = data.trim().split(":").peekable();

        match parts.peek().map(|str| &str[0..1]) {
            Some("#") => {
                let msg_id = parts.peek().and_then(|str| u8::from_str_radix(&str[1..3], 16).ok());
                let msg_type = parts.next().and_then(|str| MessageType::from_str(&str[4..5]).ok());
                let msg_speed = parts.next().and_then(|str| MessageSpeed::from_str(&str).ok());
                let msg_addr = parts.next().and_then(|str| u32::from_str_radix(&str, 16).ok());
                let msg_func = parts.next().and_then(|str| MessageFunc::from_str(&str).ok());
                let msg_text: String = parts.collect::<Vec<&str>>().join(":");

                if msg_id.is_some() && msg_type.is_some() && msg_addr.is_some() && msg_func.is_some() {
                    let msg = Message {
                        id: msg_id.unwrap(),
                        mtype: msg_type.unwrap(),
                        speed: msg_speed.unwrap_or(MessageSpeed::Baud1200),
                        addr: msg_addr.unwrap(),
                        func: msg_func.unwrap(),
                        text: msg_text
                    };

                    let next_id = (msg.id as u16 + 1) % 256;
                    self.scheduler.enqueue(msg);
                    self.send(&*format!("#{:02x} +", next_id));
                }
                else {
                    error!("Malformed message received: {}", data);
                    self.ack(AckStatus::Error);
                }
            },
            Some("2") => {
                if let Some(ident) = parts.nth(1) {
                    self.send(&*format!("2:{}:{:04x}", ident, 0));
                    self.ack(AckStatus::Success);
                }
                else {
                    self.ack(AckStatus::Error);
                }
            },
            Some("3") => {
                self.ack(AckStatus::Success);
            },
            Some("4") => {
                self.ack(AckStatus::Success);
            },
            other => {
                error!("Unknown message received from server (type: {})", other.unwrap_or("None"));
            }
        }
    }

    fn send(&mut self, data: &str) {
        info!("Response: {}", data);
        self.writer.write(data.as_bytes()).unwrap();
        self.writer.write(b"\r\n").unwrap();
        self.writer.flush().unwrap();
    }

    fn ack(&mut self, status: AckStatus) {
        let response = match status {
            AckStatus::Success => b"+\r\n",
            AckStatus::Error => b"-\r\n",
            AckStatus::Retry => b"%\r\n"
        };
        self.writer.write(response).unwrap();
        self.writer.flush().unwrap();
    }
}
