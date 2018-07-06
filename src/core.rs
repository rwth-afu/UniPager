use std::io::{self, Result};
use std::net::ToSocketAddrs;
use std::collections::HashMap;

use hyper;
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite};
use futures::future::Future;
use futures::{self, done, IntoFuture, Stream};
use futures::future::{err, ok};
use serde_json::{self, Value};

use config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub port: u16,
    pub reachable: bool,
    pub last_seen: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub timeslots: Vec<bool>,
    pub nodes: HashMap<String, Node>
}

pub fn bootstrap(config: &Config) -> impl Future<Item=BootstrapResponse,
                                                 Error=io::Error> {
    /*
    if config.master.call.len() == 0 {
        error!("No callsign configured.");
        err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No callsign configured"
        ))
    } else if config.master.auth.len() == 0 {
        error!("No auth key configured.");
        err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No auth key configured"
        ))
    } else {
    */
        info!("Connecting to {}:{}...", config.master.server, config.master.port);

        let client = hyper::Client::new();

        let url = format!("http://{}:{}/api/transmitters/bootstrap",
                          config.master.server,
                          config.master.port);

        let request = hyper::Request::builder()
            .method("POST")
            .uri(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&json!({
                "callsign": config.master.call,
                "auth_key": config.master.auth,
                "software": {
                    "name": "UniPager",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })).unwrap().into()).unwrap();

    client.request(request)
        .and_then(|res| {
            res.into_body().concat2()
        })
        .or_else(|_| {
            err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "No callsign configured"
                ))
        })
        .and_then(|body| {
            done(serde_json::from_slice(&body)
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "No callsign configured"
                    )
                }))
        })

}
