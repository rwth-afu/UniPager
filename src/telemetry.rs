use std::sync::{Mutex, RwLock};
use std::time::Duration;

use chrono::prelude::*;
use async_std::prelude::*;
use async_std::stream::interval;
use tokio::runtime::Runtime;

use crate::event::{Event, EventHandler};
use crate::queue::NUM_PRIORITIES;
use crate::timeslots::TimeSlots;

lazy_static! {
    pub static ref TELEMETRY: RwLock<Telemetry> = RwLock::new(Telemetry::default());
    pub static ref EVENT_HANDLER: Mutex<Option<EventHandler>> = Mutex::new(None);
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub port: u16,
    pub connected: bool,
    pub connected_since: Option<DateTime<Utc>>
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct Ntp {
    pub synced: bool,
    pub offset: isize,
    pub servers: Vec<String>
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct Messages {
    pub queued: [usize; NUM_PRIORITIES],
    pub sent: [usize; NUM_PRIORITIES]
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TransmitterSoftware {
    pub name: String,
    pub version: String
}

impl Default for TransmitterSoftware {
    fn default() -> Self {
        TransmitterSoftware {
            name: "UniPager".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned()
        }
    }
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct Config {
    pub timeslots: [bool; 16],
    pub software: TransmitterSoftware
}

#[derive(Default, Debug, Serialize, Clone)]
pub struct Hardware {
    pub platform: String
}

#[derive(Default, Debug, Serialize, Clone)]
pub struct Telemetry {
    pub onair: bool,
    pub timeslots: TimeSlots,
    pub node: Node,
    pub ntp: Ntp,
    pub messages: Messages,
    pub config: Config,
    pub hardware: Hardware
}

pub fn get() -> Telemetry {
    TELEMETRY.read().unwrap().clone()
}

pub fn start(runtime: &Runtime, event_handler: EventHandler) {
    *EVENT_HANDLER.lock().unwrap() = Some(event_handler.clone());

    runtime.spawn(async move {
        let mut interval = interval(Duration::from_secs(30));

        while let Some(_) = interval.next().await {
            let telemetry = get();
            event_handler.publish(Event::TelemetryUpdate(telemetry));
        }
    });
}

macro_rules! telemetry_update {
    ( $( $key:ident: $updater:expr),* ) => ({
        let mut telemetry = $crate::telemetry::TELEMETRY.write().unwrap();
        $(
            let old = telemetry.$key.clone();
            $updater(&mut telemetry.$key);

            if telemetry.$key != old {
                // Send an update to connected frontend clients
                let res = $crate::telemetry::EVENT_HANDLER.lock().unwrap();
                if let Some(ref event_handler) = *res {
                    event_handler.publish(
                        $crate::event::Event::TelemetryPartialUpdate(
                            json!({stringify!($key): telemetry.$key})
                        )
                    );
                }
            }
        )*
    });
}

macro_rules! telemetry {
    ( $( $key:ident: $value:expr),* ) => ({
        let mut telemetry = $crate::telemetry::TELEMETRY.write().unwrap();
        $(
            // Update only if the value has changed
            if telemetry.$key != $value {
                telemetry.$key = $value;

                // Send an update to connected frontend clients
                let res = $crate::telemetry::EVENT_HANDLER.lock().unwrap();
                if let Some(ref event_handler) = *res {
                    event_handler.publish(
                        $crate::event::Event::TelemetryPartialUpdate(
                            json!({stringify!($key): $value})
                        )
                    );
                }
            }
        )*
    });
}
