use std::collections::HashMap;
use std::io;
use std::time::Duration;

use futures::stream::StreamExt;
use async_std::stream::interval;
use tokio::runtime::Runtime;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::event::EventHandler;

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

pub async fn bootstrap(config: &Config) -> Result<BootstrapResponse, io::Error> {
    if config.master.call.len() == 0 {
        error!("No callsign configured.");
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No callsign configured"
        ))
    } else if config.master.auth.len() == 0 {
        error!("No auth key configured.");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No auth key configured"
        ))
    } else {
        info!(
            "Connecting to {}:{}...",
            config.master.server,
            config.master.port
        );

        let url = format!(
            "http://{}:{}/transmitters/_bootstrap",
            config.master.server,
            config.master.port
        );

        reqwest::Client::new()
            .post(&url)
            .json(&json!({
                "callsign": config.master.call,
                "auth_key": config.master.auth,
                "software": {
                    "name": "UniPager",
                    "version": env!("CARGO_PKG_VERSION")
                }})
            ).send().await.unwrap().json().await.map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Bootstrap data parsing failed"
                )})
    }
}

pub async fn heartbeat(config: &Config) -> Result<HeartbeatResponse, io::Error> {
    debug!("Sending Heartbeat");

    let url = format!(
        "http://{}:{}/transmitters/_heartbeat",
        config.master.server,
        config.master.port
    );


    reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "callsign": config.master.call,
            "auth_key": config.master.auth,
        })
        ).send().await.unwrap().json().await.map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bootstrap data parsing failed"
            )})
}

pub fn start(runtime: &Runtime, config: &Config, _event_handler: EventHandler) {
    let config = config.clone();

    runtime.spawn(async move {
        let mut interval = interval(Duration::from_secs(60));

        while let Some(_now) = interval.next().await {
            heartbeat(&config).await;
        }
    });
}
