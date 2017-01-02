use ws;
use serde_json;
use config::Config;

#[derive(Debug, Deserialize)]
enum Request {
    GetConfig,
    SetConfig,
    SendMessage { addr: u32, message: String }
}

#[derive(Debug, Serialize)]
enum Response {
    Config(Config)
}

struct Server {
    out: ws::Sender,
}

impl Server {
    fn send_response(&mut self, res: Response) -> ws::Result<()> {
        let data = serde_json::to_string(&res).unwrap();
        self.out.send(data)
    }

    fn decode_request(&mut self, msg: &ws::Message) -> Option<Request> {
        msg.as_text().ok().and_then(|text| serde_json::from_str(&text).ok())
    }
}

impl ws::Handler for Server {
    fn on_open(&mut self, handshake: ws::Handshake) -> ws::Result<()> {
        self.send_response(Response::Config(Config::default()))
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("Server got message '{}'. ", msg);
        let req = self.decode_request(&msg);
        println!("Decoded Request: {:?}", req);
        self.out.send(msg)
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }
}

pub fn run() {
    ws::listen("0.0.0.0:2794", |out| {
        Server { out: out }
    }).unwrap();
}
