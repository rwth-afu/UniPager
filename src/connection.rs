use std::time::Duration;

use lapin::{self, message::Delivery, options::*, types::FieldTable,
            BasicProperties, Connection, ConnectionProperties, Channel};
use futures::{future::FutureExt, stream::StreamExt, select, pin_mut};
use futures_timer::Delay;
use tokio::{self, runtime::Runtime};
use tokio_amqp::*;
use serde_json;

use crate::config::Config;
use crate::core;
use crate::event::{self, Event, EventHandler, EventReceiver};
use crate::message::Message;
use crate::telemetry;
use crate::timeslots::TimeSlots;

struct CoreConnection {
    config: Config,
    event_handler: EventHandler,
    event_receiver: EventReceiver,
    routing_key: String,
    telemetry_routing_key: String,
    restart: bool
}

impl CoreConnection {
    pub fn new(config: Config, event_handler: EventHandler) -> CoreConnection {
        let (tx, rx) = event::channel();
        event_handler.publish(Event::RegisterConnection(tx));

        CoreConnection {
            config,
            event_handler,
            event_receiver: rx,
            routing_key: "".to_owned(),
            telemetry_routing_key: "".to_owned(),
            restart: true
        }
    }

    pub async fn start(&mut self) {
        loop {
            self.handle_events(None, None).await;

            if let Ok(response) = core::bootstrap(&self.config).await {
                info!("Bootstrap successful. Found {} nodes.", response.nodes.len());

                let timeslots = TimeSlots::from_vec(response.timeslots);
                self.event_handler.publish(Event::TimeslotsUpdate(timeslots));

                info!("Timeslots updated: {:?}", timeslots);

                self.run().await;
                warn!("RabbitMQ Connection lost.");
            }
            else {
                error!(
                    "Bootstrap connection failed. Retrying in {} Seconds...",
                    self.config.master.reconnect_timeout
                );
                Delay::new(Duration::from_secs(self.config.master.reconnect_timeout)).await;
            }
        }
    }

    async fn run(&mut self) -> Result<(), lapin::Error> {
        info!("Starting RabbitMQ connection.");
        let call = self.config.master.call.to_owned().to_ascii_lowercase();
        let user = format!("tx-{}", &call).to_owned();

        self.routing_key = call.to_owned();
        self.telemetry_routing_key = format!("transmitter.{}", call).to_owned();
        let auth_key = self.config.master.auth.to_owned();

        let host = &self.config.master.server;
        let port = 5672;

        let addr = format!("amqp://{}:{}@{}:{}/%2f", user, auth_key, &**host, port);

        telemetry_update!(node: &|node: &mut telemetry::Node| {
            *node = telemetry::Node {
                name: host.to_owned(),
                port: port,
                connected: false,
                connected_since: None
            };
        });

        info!("Connecting to {}:{}...", host, port);

        let conn = Connection::connect(
            &addr,
            ConnectionProperties::default().with_tokio()
        ).await?;

        let channel = conn.create_channel().await?;

        let queue = channel
            .queue_declare(
                &call,
                QueueDeclareOptions::default(),
                FieldTable::default()
            )
            .await?;

        channel
            .queue_bind(
                &*queue.name().to_string(),
                "dapnet.local_calls",
                &*self.routing_key,
                QueueBindOptions::default(),
                FieldTable::default()
            )
            .await?;

        let mut consumer = channel
            .basic_consume(
                &*queue.name().to_string(),
                "consumer",
                BasicConsumeOptions::default(),
                FieldTable::default()
            )
            .await?;

        info!("Connected to RabbitMQ. Listening for incoming calls.");

        telemetry_update!(node: &|node: &mut telemetry::Node| {
            node.connected = true;
            node.connected_since = Some(::chrono::Utc::now());
        });

        loop {
            let next_delivery = consumer.next().fuse();
            let next_event = self.event_receiver.next().fuse();

            pin_mut!(next_delivery, next_event);

            select! {
                delivery = next_delivery => {
                    if let Some(Ok(delivery)) = delivery {
                        self.handle_delivery(delivery, &*channel).await?;
                    }
                    else {
                        break;
                    }
                },
                event = next_event => {
                    if let Some(event) = event {
                        self.handle_event(event, Some(&*conn), Some(&*channel)).await?;
                    }
                    else {
                        break;
                    }
                },
                complete => {
                    break;
                }
            }
        }

        telemetry_update!(node: &|node: &mut telemetry::Node| {
            node.connected = false;
            node.connected_since = None;
        });

        Ok(())
    }

    async fn handle_delivery(&mut self, delivery: Delivery, channel: &Channel) -> Result<(), lapin::Error> {
        let msg: Option<Message> = ::std::str::from_utf8(&delivery.data)
            .ok()
            .and_then(|str| serde_json::from_str(&str).ok());

        if let Some(msg) = msg {
            info!("Message received: {:?}", msg);
            self.event_handler.publish(Event::MessageReceived(msg));
        }
        else {
            warn!("Could not decode incoming message");
        }

        channel
            .basic_ack(
                delivery.delivery_tag,
                BasicAckOptions::default()
            )
            .map(|_| ())
            .await;

        Ok(())
    }

    async fn handle_events(&mut self, conn: Option<&Connection>, channel: Option<&Channel>) -> Result<(), lapin::Error> {
        while let Ok(Some(event)) = self.event_receiver.try_next() {
            self.handle_event(event, conn, channel).await?;
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: Event, conn: Option<&Connection>, channel: Option<&Channel>) -> Result<(), lapin::Error> {
        match event {
            Event::TelemetryUpdate(telemetry) => {
                if let Some(channel) = channel {
                    self.send_telemetry(channel, serde_json::to_vec(&telemetry).unwrap()).await?;
                }
            }
            Event::TelemetryPartialUpdate(telemetry) => {
                if let Some(channel) = channel {
                    self.send_telemetry(channel, serde_json::to_vec(&telemetry).unwrap()).await?;
                }
            }
            Event::ConfigUpdate(new_config) => {
                self.restart = true;
                self.config = new_config;
                if let Some(conn) = conn {
                    conn.close(0, "reconfig").await?;
                }
            }
            Event::Restart => {
                self.restart = true;
                if let Some(conn) = conn {
                    conn.close(0, "restart").await?;
                }
            }
            Event::Shutdown => {
                self.restart = false;
                if let Some(conn) = conn {
                    conn.close(0, "shutdown").await?;
                }
            }
            _ => {}
        };
        Ok(())
    }

    async fn send_telemetry(&self, channel: &Channel, data: Vec<u8>) -> Result<(), lapin::Error> {
        channel
            .basic_publish(
                "dapnet.telemetry",
                &*self.telemetry_routing_key,
                BasicPublishOptions::default(),
                data,
                BasicProperties::default()
            ).map(|_| Ok(())).await
    }
}

pub fn start(runtime: &Runtime, config: &Config, event_handler: EventHandler) {
    let mut conn = CoreConnection::new(config.clone(), event_handler.clone());
    runtime.spawn(async move { conn.start().await; });
}
