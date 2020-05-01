use std::sync::mpsc;

use futures;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use tokio::runtime::Runtime;
use serde_json;

use crate::telemetry::Telemetry;
use crate::config::Config;
use crate::message::Message;
use crate::timeslots::{TimeSlot, TimeSlots};

#[derive(Clone, Debug)]
pub enum Event {
    TelemetryUpdate(Telemetry),
    TelemetryPartialUpdate(serde_json::value::Value),
    Timeslot(TimeSlot),
    TimeslotsUpdate(TimeSlots),
    ConfigUpdate(Config),
    MessageReceived(Message),
    RegisterConnection(EventSender),
    RegisterWebsocket(EventSender),
    RegisterScheduler(mpsc::Sender<Event>),
    RegisterMain(EventSender),
    Log(u8, String),
    Shutdown
}

pub type EventReceiver = UnboundedReceiver<Event>;
pub type EventSender = UnboundedSender<Event>;

#[derive(Clone)]
pub struct EventHandler(EventSender);

impl EventHandler {
    pub fn publish(&self, event: Event) {
        self.0.unbounded_send(event).unwrap();
    }
}

#[derive(Default)]
pub struct EventDispatcher {
    pub connection: Option<EventSender>,
    pub scheduler: Option<mpsc::Sender<Event>>,
    pub websocket: Option<EventSender>,
    pub main: Option<EventSender>
}

impl EventDispatcher {

}

pub fn channel() -> (EventSender, EventReceiver) {
    futures::channel::mpsc::unbounded()
}

pub fn start(runtime: &Runtime) -> EventHandler {
    let (event_tx, event_rx) = channel();

    runtime.spawn(async move {
        let mut dispatcher = EventDispatcher::default();
        let mut event_rx = event_rx;

        while let Some(event) = event_rx.next().await {
            match event {
                Event::RegisterConnection(tx) => {
                    dispatcher.connection = Some(tx);
                },
                Event::RegisterWebsocket(tx) => {
                    dispatcher.websocket = Some(tx);
                },
                Event::RegisterScheduler(tx) => {
                    dispatcher.scheduler = Some(tx);
                },
                Event::RegisterMain(tx) => {
                    dispatcher.main = Some(tx);
                },
                Event::TelemetryUpdate(_) |
                Event::TelemetryPartialUpdate(_) => {
                    dispatcher.websocket.as_ref().map(|tx| {
                        tx.unbounded_send(event.clone()).ok();
                    });
                    dispatcher.connection.as_ref().map(|tx| {
                        tx.unbounded_send(event).ok();
                    });
                }
                Event::Log(_, _) | Event::Timeslot(_) => {
                    dispatcher.websocket.as_ref().map(|tx| {
                        tx.unbounded_send(event).ok();
                    });
                }
                Event::MessageReceived(_) => {
                    dispatcher.scheduler.as_ref().map(|tx| {
                        tx.send(event.clone()).ok();
                    });
                    dispatcher.websocket.as_ref().map(|tx| {
                        tx.unbounded_send(event).ok();
                    });
                }
                Event::TimeslotsUpdate(_) => {
                    dispatcher.scheduler.as_ref().map(|tx| {
                        tx.send(event.clone()).ok();
                    });
                }
                Event::Shutdown => {
                    dispatcher.main.as_ref().map(|tx| {
                        tx.unbounded_send(event.clone()).ok();
                    });
                }
                _ => {}
            };
        }
    });

    EventHandler(event_tx)
}
