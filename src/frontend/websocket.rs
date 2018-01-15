use serde_json;
use std::sync::mpsc::Sender;
use std::thread;
use ws;

use frontend::{Request, Response};

struct Server {
    ws: ws::Sender,
    tx: Sender<Request>,
    password: Option<String>,
    auth: bool
}

impl ws::Handler for Server {
    fn on_open(&mut self, hs: ws::Handshake) -> ws::Result<()> {
        if hs.peer_addr.map(|addr| addr.ip().is_loopback()).unwrap_or(false) {
            self.password = None;
            self.auth = true;
        }
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let req = msg.as_text().ok().and_then(
            |text| serde_json::from_str(text).ok()
        );

        if let Some(req) = req {
            if let Request::Authenticate(ref pass) = req {
                self.auth = self.password.as_ref()
                    .map(|p| pass == p)
                    .unwrap_or(true);

                let res = Response::Authenticated(self.auth);
                let data = serde_json::to_string(&res).unwrap();
                self.ws.send(data).unwrap();
            }
            else if self.auth {
                self.tx.send(req).unwrap();
            }
            else {
                let res = Response::Authenticated(false);
                let data = serde_json::to_string(&res).unwrap();
                self.ws.send(data).unwrap();
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Responder(ws::Sender);

impl Responder {
    pub fn send(&self, res: Response) {
        let data = serde_json::to_string(&res).unwrap();
        self.0.send(data).unwrap();
    }
}

pub fn create(tx: Sender<Request>, pass: Option<&str>) -> Responder {
    let pass = pass.map(str::to_owned);
    let socket = ws::Builder::new()
        .build(move |ws| {
            Server {
                tx: tx.clone(),
                password: pass.clone(),
                auth: !pass.is_some(),
                ws: ws
            }
        })
        .unwrap();

    let broadcaster = socket.broadcaster();

    thread::spawn(move || socket.listen("0.0.0.0:8055").unwrap());

    Responder(broadcaster)
}
