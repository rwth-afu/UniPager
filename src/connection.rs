use std::net::TcpStream;
use std::io::{BufReader, BufWriter, BufRead, Write};
use std::str;

pub struct Connection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>
}

enum AckStatus {
    Success,
    Error,
    Retry
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let stream1 = stream.try_clone().unwrap();
        let stream2 = stream1.try_clone().unwrap();
        Connection {
            stream: stream,
            reader: BufReader::new(stream1),
            writer: BufWriter::new(stream2)
        }
    }

    pub fn run(&mut self) {
        self.writer.write(b"[SDRPager v1.2-SCP-#2345678]\r\n").unwrap();
        self.writer.flush();

        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).unwrap();
            self.handle(&*line);
        }
    }

    fn handle(&mut self, data: &str) {
        let mut parts = data.trim().split(":");
        let mtype = parts.next().unwrap_or("");
        info!("Received data: {}", data);

        match mtype {
            "#" => {

            },
            "2" => {
                if let Some(ident) = parts.next() {
                    self.send(&*format!("2:{}:{:04x}", ident, 0));
                    self.ack(AckStatus::Success);
                }
                else {
                    self.ack(AckStatus::Error);
                }
            },
            "3" => {
                self.ack(AckStatus::Success);
            },
            "4" => {
                self.ack(AckStatus::Success);
            },
            _ => {
                error!("unknown message type");
            }
        }
    }

    fn send(&mut self, data: &str) {
        info!("Response: {}", data);
        self.writer.write(data.as_bytes());
        self.writer.write(b"\r\n");
        self.writer.flush();
    }

    fn ack(&mut self, status: AckStatus) {
        let response = match status {
            AckStatus::Success => b"+\r\n",
            AckStatus::Error => b"-\r\n",
            AckStatus::Retry => b"%\r\n"
        };
        self.writer.write(response);
        self.writer.flush();
    }
}
