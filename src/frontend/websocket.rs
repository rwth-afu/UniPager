use std::sync::mpsc::Sender;
use std::thread;
use serde_json;
use ws;

use frontend::{Request, Response};

struct Server {
    tx: Sender<Request>
}

impl ws::Handler for Server {
    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let req = msg.as_text().ok()
            .and_then(|text| serde_json::from_str(&text).ok());

        if let Some(req) = req {
            self.tx.send(req).unwrap();
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

pub fn create(tx: Sender<Request>) -> Responder {
    let socket = ws::Builder::new().build(move |_| {
        let tx = tx.clone();
        Server { tx: tx }
    }).unwrap();

    let broadcaster = socket.broadcaster();

    thread::spawn(move || socket.listen("0.0.0.0:8055").unwrap());

    return Responder(broadcaster);
}
