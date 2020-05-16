use std::{collections::HashMap, io, time::Duration};

use async_std::stream::interval;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{config::Config, event::EventHandler};

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

pub async fn bootstrap(
    config: &Config
) -> Result<BootstrapResponse, io::Error> {
    if config.master.call.len() == 0 {
        error!("No callsign configured.");
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No callsign configured"
        ))
    }
    else if config.master.auth.len() == 0 {
        error!("No auth key configured.");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No auth key configured"
        ));
    }
    else {
        info!(
            "Connecting to {}:{}...",
            config.master.server, config.master.port
        );

        let url = format!(
            "http://{}:{}/transmitters/_bootstrap",
            config.master.server, config.master.port
        );

        reqwest::Client::new()
            .post(&url)
            .json(&json!({
            "callsign": config.master.call,
            "auth_key": config.master.auth,
            "software": {
                "name": "UniPager",
                "version": env!("CARGO_PKG_VERSION")
            }}))
            .send()
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Bootstrap connection failed"
                )
            })?
            .json()
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Bootstrap data parsing failed"
                )
            })
    }
}

pub async fn heartbeat(
    config: &Config
) -> Result<HeartbeatResponse, io::Error> {
    info!("Sending Heartbeat");

    let url = format!(
        "http://{}:{}/transmitters/_heartbeat",
        config.master.server, config.master.port
    );

    reqwest::Client::new()
        .post(&url)
        .json(&json!({
            "callsign": config.master.call,
            "auth_key": config.master.auth,
        }))
        .send()
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Heartbeat connection failed"
            )
        })?
        .json()
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Heartbeat data parsing failed"
            )
        })
}

pub fn start(runtime: &Runtime, config: &Config, _event_handler: EventHandler) {
    let config = config.clone();

    runtime.spawn(async move {
        let mut interval = interval(Duration::from_secs(60));

        while let Some(_now) = interval.next().await {
            let res = heartbeat(&config).await;
            info!("Heartbeat Result: {:?}", res);
        }
    });
}
