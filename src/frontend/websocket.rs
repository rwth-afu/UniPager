use futures::sync::mpsc::UnboundedSender;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use frontend::{Request, Response};
use futures::{self, Future};
use futures::stream::Stream;
use serde_json;
use tokio;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;

use config;
use event::{self, Event, EventHandler};
use telemetry;
use timeslots::TimeSlot;

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
                /*
                restart = true;
                test = true;
                stop_conn.send(()).ok();
                scheduler.stop();
                */
            }
            Request::Authenticate(_) => {}
        }
    }
}

pub fn start(rt: &mut Runtime, pass: Option<String>, event_handler: EventHandler) {
    let (tx, rx) = event::channel();

    event_handler.publish(Event::RegisterWebsocket(tx));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8055));

    let socket = TcpListener::bind(&addr).unwrap();
    let connections = Arc::new(Mutex::new(HashMap::new()));
    let connections_rx = connections.clone();

    let server = socket
        .incoming()
        .for_each(move |stream| {
            let addr = stream.peer_addr().unwrap();
            let connections_inner = connections.clone();
            let pass = pass.to_owned();
            let event_handler = event_handler.clone();

            accept_async(stream)
                .and_then(move |ws_stream| {
                    let (tx, rx) = futures::sync::mpsc::unbounded();
                    let (sink, stream) = ws_stream.split();

                    let mut connection = Connection {
                        tx: tx.clone(),
                        password: pass.to_owned(),
                        auth: false,
                        event_handler: event_handler
                    };

                    connections_inner.lock().unwrap().insert(addr, tx);

                    let ws_reader = stream
                        .for_each(move |msg: Message| {
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
                            Ok(())
                        })
                        .map(|_| ())
                        .map_err(|_e| ());

                    let ws_writer = rx.fold(sink, |mut sink, msg| {
                        use futures::Sink;
                        let data = serde_json::to_string(&msg).unwrap();
                        let msg = Message::text(data);
                        sink.start_send(msg).unwrap();
                        Ok(sink)
                    }).map(|_| ())
                        .map_err(|_e| ());

                    let connection = ws_reader.select(ws_writer);

                    tokio::spawn(connection.then(move |_| {
                        connections_inner.lock().unwrap().remove(&addr);
                        Ok(())
                    }));

                    Ok(())
                })
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        })
        .map(|_| ())
        .map_err(|_e| ());

    rt.spawn(server);

    rt.spawn(rx.for_each(move |event| {
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
        Ok(())
    }));
}
