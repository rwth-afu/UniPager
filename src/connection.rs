use serde_json;

use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Error
};

use futures::{future::FutureExt, stream::StreamExt};
use tokio;
use tokio::runtime::Runtime;

use crate::config::Config;
use crate::core;
use crate::event::{self, Event, EventHandler};
use crate::message::Message;
use crate::telemetry;
use crate::timeslots::TimeSlots;

async fn connection(config: Config, event_handler: EventHandler) -> Result<(), Error> {
    info!("Starting RabbitMQ connection");
    let call = config.master.call.to_owned().to_ascii_lowercase();
    let user = format!("tx-{}", &call).to_owned();
    let routing_key = call.to_owned();
    let telemetry_routing_key = format!("transmitter.{}", call).to_owned();
    let auth_key = config.master.auth.to_owned();
    let host = &config.master.server;
    let port = 5672;

    let (tx, rx) = event::channel();
    event_handler.publish(Event::RegisterConnection(tx));

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

    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;

    let channel = conn.create_channel().await?;

    /*
    tokio::spawn(
        core::heartbeat
            .map(|_| warn!("Heartbeat process finished."))
            .map_err(|err| warn!("Heartbeat process failed: {:?}", err))
    );*/

    let queue = channel.queue_declare(
        &call,
        QueueDeclareOptions::default(),
        FieldTable::default()
    ).await?;

    channel.queue_bind(
        &*queue.name().to_string(),
        "dapnet.local_calls",
        &*routing_key,
        QueueBindOptions::default(),
        FieldTable::default()
    ).await?;

    let consumer = channel.basic_consume(
        &*queue.name().to_string(),
        "consumer",
        BasicConsumeOptions::default(),
        FieldTable::default()
    ).await?;

    info!("Listening for incoming calls.");

    telemetry_update!(node: &|node: &mut telemetry::Node| {
        node.connected = true;
        node.connected_since = Some(::chrono::Utc::now());
    });

    let channel2 = channel.clone();

    tokio::spawn(async move {
        let mut rx = rx;
        while let Some(event) = rx.next().await {
            let data = match event
            {
                Event::TelemetryUpdate(telemetry) => {
                    serde_json::to_vec(&telemetry).unwrap()
                }
                Event::TelemetryPartialUpdate(telemetry) => {
                    serde_json::to_vec(&telemetry).unwrap()
                }
                _ => Vec::new(),
            };

            channel2
                .basic_publish(
                    "dapnet.telemetry",
                    &*telemetry_routing_key,
                    BasicPublishOptions::default(),
                    data,
                    BasicProperties::default()
                ).map(|_| ()).await;
        }
    });

    // Consume the messages
    consumer.for_each(move |message| {
        use std::str::from_utf8;

        let message = message.expect("Invalid message");

        let msg: Option<Message> =
            from_utf8(&message.data).ok().and_then(|str| {
                serde_json::from_str(&str).ok()
            });

        if let Some(msg) = msg {
            info!("Message received: {:?}", msg);
            event_handler.publish(Event::MessageReceived(msg));
        }
        else {
            warn!("Could not decode incoming message")
        }
        channel.basic_ack(message.delivery_tag, BasicAckOptions::default()).map(|_| ())
    }).await;

    telemetry_update!(node: &|node: &mut telemetry::Node| {
        node.connected = false;
        node.connected_since = None;
    });

    warn!("RabbitMQ connection lost.");

    Ok(())
}

pub fn start(runtime: &Runtime, config: &Config, event_handler: EventHandler) {
    let config = config.clone();
    let event_handler = event_handler.clone();

    runtime.spawn(
        async move {
            let config = config.clone();
            let event_handler = event_handler.clone();
            let response = core::bootstrap(&config).await.unwrap();
            let timeslots = TimeSlots::from_vec(response.timeslots);
            event_handler.publish(Event::TimeslotsUpdate(timeslots));
            connection(config, event_handler.clone()).await
        }
    );
}
