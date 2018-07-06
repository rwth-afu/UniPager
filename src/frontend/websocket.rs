use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use futures::sync::mpsc::UnboundedSender;

use frontend::{Request, Response};
use futures::{self, Future};
use futures::stream::Stream;
use serde_json;
use tokio;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;

struct Connection {
    tx: UnboundedSender<Response>,
    password: Option<String>,
    auth: bool,
}

impl Connection {
    fn handle(&mut self, req: &Request) {
        if let Request::Authenticate(ref pass) = req {
            self.auth = self.password.as_ref()
                .map(|p| pass == p)
                .unwrap_or(true);

            let res = Response::Authenticated(self.auth);
            self.tx.send(res).unwrap();
        }
        else if self.auth {
            //self..send(req).unwrap();
        }
        else {
            let res = Response::Authenticated(false);
            self.tx.send(res).unwrap();
        }
    }
}

pub fn server(pass: Option<String>) -> impl Future<Item = (), Error = ()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8055));

    let socket = TcpListener::bind(&addr).unwrap();
    let connections = Arc::new(Mutex::new(HashMap::new()));

    socket
        .incoming()
        .for_each(move |stream| {
            let addr = stream.peer_addr().unwrap();
            let connections_inner = connections.clone();
            let pass = pass.to_owned();

            accept_async(stream)
                .and_then(move |ws_stream| {
                    let (tx, rx) = futures::sync::mpsc::unbounded();
                    let (sink, stream) = ws_stream.split();

                    let mut connection = Connection {
                        tx: tx.clone(),
                        password: pass.to_owned(),
                        auth: false
                    };

                    connections_inner.lock().unwrap().insert(addr, tx);

                    let ws_reader =
                        stream
                        .for_each(move |msg: Message| {
                            msg.to_text().ok()
                                .and_then(|str| serde_json::from_str(&str).ok())
                                .map(|req| connection.handle(&req))
                                .or_else(|| {
                                    warn!("Received unreadable websocket request.");
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
                    })
                        .map(|_| ())
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
        .map_err(|_e| ())
}
