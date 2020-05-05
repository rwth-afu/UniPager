use futures::channel::mpsc::UnboundedSender;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio;
use serde_json;
use futures;
use tungstenite::protocol::Message;
use futures_util::{StreamExt, TryStreamExt};

use crate::frontend::{Request, Response};
use crate::config;
use crate::event::{self, Event, EventHandler};
use crate::telemetry;
use crate::timeslots::TimeSlot;

type PeerMap = Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Response>>>>;

struct Connection {
    tx: UnboundedSender<Response>,
    password: Option<String>,
    auth: bool,
    event_handler: EventHandler
}

impl Connection {
    fn handle(&mut self, req: &Request) {
        if let Request::Authenticate(ref pass) = req {
            self.auth =
                self.password.as_ref().map(|p| pass == p).unwrap_or(true);

            let res = Response::Authenticated(self.auth);
            self.tx.unbounded_send(res).ok();
        }
        else if self.auth {
            self.handle_request(req);
        }
        else {
            let res = Response::Authenticated(false);
            self.tx.unbounded_send(res).ok();
        }
    }

    fn handle_request(&mut self, req: &Request) {
        match req
        {
            Request::SetConfig(new_config) => {
                let config = new_config.clone();
                config::set(&config);
                self.event_handler.publish(Event::ConfigUpdate(config));
            }
            Request::DefaultConfig => {
                let config = config::Config::default();
                config::set(&config);
                self.event_handler.publish(Event::ConfigUpdate(config));
            }
            Request::SendMessage(msg) => {
                let msg = msg.clone();
                self.event_handler.publish(Event::MessageReceived(msg));
            }
            Request::GetConfig => {
                self.tx.unbounded_send(Response::Config(config::get())).ok();
            }
            Request::GetVersion => {
                let version = env!("CARGO_PKG_VERSION").to_string();
                self.tx.unbounded_send(Response::Version(version)).ok();
            }
            Request::GetTelemetry => {
                self.tx
                    .unbounded_send(Response::Telemetry(telemetry::get()))
                    .ok();
            }
            Request::GetTimeslot => {
                self.tx
                    .unbounded_send(Response::Timeslot(TimeSlot::current()))
                    .ok();
            }
            Request::Test => {
                info!("Initiating test procedure...");
                self.event_handler.publish(Event::Test);
            },
            Request::Restart => {
                self.event_handler.publish(Event::Restart);
            },
            Request::Shutdown => {
                self.event_handler.publish(Event::Shutdown);
            },
            Request::Authenticate(_) => {}
        }
    }
}

async fn handle_connection(connections: PeerMap, pass: Option<String>, event_handler: EventHandler, stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();

    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

    let (tx, rx) = futures::channel::mpsc::unbounded();
    let (sink, stream) = ws_stream.split();

    let mut connection = Connection {
        tx: tx.clone(),
        password: pass.to_owned(),
        auth: false,
        event_handler: event_handler
    };

    connections.lock().unwrap().insert(addr, tx);

    let ws_reader = stream.try_for_each(|msg| {
        msg.to_text()
            .ok()
            .and_then(|str| serde_json::from_str(&str).ok())
            .map(|req| connection.handle(&req))
            .or_else(|| {
                warn!(
                    "Received unreadable websocket request."
                );
                None
            });

        futures::future::ok(())
    });

    let ws_writer = rx.map(|msg| {
        let data = serde_json::to_string(&msg).unwrap();
        Ok(Message::text(data))
    }).forward(sink);

    futures::future::select(ws_reader, ws_writer).await;
    connections.lock().unwrap().remove(&addr);
}

pub fn start(runtime: &Runtime, pass: Option<String>, event_handler: EventHandler) {
    let (tx, rx) = event::channel();

    event_handler.publish(Event::RegisterWebsocket(tx));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8055));

    let connections: PeerMap = Arc::new(Mutex::new(HashMap::new()));
    let connections_rx = connections.clone();

    runtime.spawn(async move {
        let mut rx = rx;

        while let Some(event) = rx.next().await {
            for (_, connection) in connections_rx.lock().unwrap().iter() {
                let response = match event.clone() {
                    Event::TelemetryUpdate(telemetry) => {
                        Some(Response::Telemetry(telemetry))
                    }
                    Event::TelemetryPartialUpdate(value) => {
                        Some(Response::TelemetryUpdate(value))
                    }
                    Event::MessageReceived(msg) => {
                        Some(Response::Message(msg))
                    }
                    Event::Timeslot(timeslot) => {
                        Some(Response::Timeslot(timeslot))
                    }
                    Event::Log(level, message) => {
                        Some(Response::Log(level, message))
                    }
                    _ => None
                };

                if let Some(response) = response {
                    connection.unbounded_send(response).ok();
                }
            }
        }
    });

    runtime.spawn(async move {
        let mut socket = TcpListener::bind(&addr).await.unwrap();

        while let Ok ((stream, _)) = socket.accept().await {
            tokio::spawn(
                handle_connection(connections.clone(), pass.to_owned(),
                                  event_handler.clone(), stream)
            );
        }

        info!("Shutting down websocket server!");
    });
}
