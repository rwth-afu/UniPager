use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use futures::{Stream, done};
use futures::future::Future;
use futures::future::err;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

use hyper;
use serde_json;

use config::Config;
use event::EventHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub host: String,
    pub reachable: bool,
    pub last_seen: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub timeslots: Vec<bool>,
    pub nodes: HashMap<String, Node>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String
}

pub fn bootstrap(config: &Config)
    -> impl Future<Item = BootstrapResponse, Error = io::Error> {
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
    info!(
        "Connecting to {}:{}...",
        config.master.server,
        config.master.port
    );

    let client = hyper::Client::new();

    let url = format!(
        "http://{}:{}/transmitters/_bootstrap",
        config.master.server,
        config.master.port
    );

    let request = hyper::Request::builder()
        .method("POST")
        .uri(url)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&json!({
                "callsign": config.master.call,
                "auth_key": config.master.auth,
                "software": {
                    "name": "UniPager",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })).unwrap()
                .into()
        )
        .unwrap();

    client
        .request(request)
        .and_then(|res| res.into_body().concat2())
        .or_else(|_| {
            err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bootstrap http request failed"
            ))
        })
        .and_then(|body| {
            done(serde_json::from_slice(&body).map_err(|_| {
                println!("{:?}", body);
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Bootstrap data parsing failed"
                )
            }))
        })
}

pub fn heartbeat(config: &Config)
             -> impl Future<Item = HeartbeatResponse, Error = io::Error> {
    debug!("Sending Heartbeat");

    let client = hyper::Client::new();

    let url = format!(
        "http://{}:{}/transmitters/_heartbeat",
        config.master.server,
        config.master.port
    );

    let request = hyper::Request::builder()
        .method("POST")
        .uri(url)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&json!({
                "callsign": config.master.call,
                "auth_key": config.master.auth
            })).unwrap()
                .into()
        )
        .unwrap();

    client
        .request(request)
        .and_then(|res| res.into_body().concat2())
        .or_else(|_| {
            err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No callsign configured"
            ))
        })
        .and_then(|body| {
            done(serde_json::from_slice(&body).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "No callsign configured"
                )
            }))
        })
}

pub fn start(rt: &mut Runtime, config: &Config, _event_handler: EventHandler) {
    let timer = Interval::new(Instant::now(), Duration::from_secs(60));
    let config = config.clone();

    let updater = timer
        .map_err(|_| ())
        .for_each(move |_| {
            heartbeat(&config)
                .map_err(|_| {
                    warn!("Heartbeat failed.");
                    ()
                })
                .and_then(|res| {
                    debug!("Heartbeat Response: {:?}", res);
                    Ok(())
                })
        });

    rt.spawn(updater);
}
