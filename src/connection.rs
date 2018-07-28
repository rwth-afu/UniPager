use std::net::ToSocketAddrs;

use serde_json;

use futures::Stream;
use futures::future::Future;

use tokio;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

use lapin::channel::{BasicConsumeOptions, BasicProperties,
                     BasicPublishOptions, QueueBindOptions,
                     QueueDeclareOptions};
use lapin::client::{Client, ConnectionOptions};
use lapin::types::FieldTable;

use config::Config;
use event::{self, Event, EventHandler};
use message::Message;
use telemetry;

pub fn start(rt: &mut Runtime, config: &Config, event_handler: EventHandler) {
    let call = config.master.call.to_owned().to_ascii_lowercase();
    let user = format!("tx-{}", &call).to_owned();
    let routing_key = call.to_owned();
    let telemetry_routing_key = format!("transmitter.{}", call).to_owned();
    let auth_key = config.master.auth.to_owned();
    let host = &config.master.server;
    let port = 5672;

    let (tx, rx) = event::channel();
    event_handler.publish(Event::RegisterConnection(tx));

    let addr = (&**host, port).to_socket_addrs().unwrap().next().expect(
        "Cannot resolve hostname"
    );

    telemetry_update!(node: &|node: &mut telemetry::Node| {
        *node = telemetry::Node {
            name: host.to_owned(),
            ip: addr.ip().to_string(),
            port: port,
            connected: false,
            connected_since: None
        };
    });

    info!("Connecting to {}:{}...", host, port);

    let connection = TcpStream::connect(&addr)
        .and_then(|stream| {
            Client::connect(
                stream,
                ConnectionOptions {
                    username: user,
                    password: auth_key,
                    vhost: "/".to_owned(),
                    frame_max: 0,
                    heartbeat: 30
                }
            )
        })
        .and_then(|(client, heartbeat)| {
            tokio::spawn(heartbeat.map_err(|_| ()));
            client.create_channel()
        })
        .and_then(move |channel| {
            // Declare queue
            channel
                .queue_declare(
                    &call,
                    QueueDeclareOptions::default(),
                    FieldTable::new()
                )
                .map(|queue| (channel, queue))
        })
        .and_then(move |(channel, queue)| {
            // Bind queue to exchange
            channel
                .queue_bind(
                    &queue.name(),
                    "dapnet.calls",
                    &*routing_key,
                    QueueBindOptions::default(),
                    FieldTable::new()
                )
                .map(|_| (channel, queue))
        })
        .and_then(|(channel, queue)| {
            // Create a consumer
            channel
                .basic_consume(
                    &queue,
                    "consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::new()
                )
                .map(move |stream| (channel, stream))
        })
        .and_then(move |(channel, stream)| {
            info!("Listening for incoming calls.");

            telemetry_update!(node: &|node: &mut telemetry::Node| {
            node.connected = true;
            node.connected_since = Some(::chrono::Utc::now());
        });

            let channel2 = channel.clone();

            tokio::spawn(rx.for_each(move |event| {
                let data = match event
                {
                    Event::TelemetryUpdate(telemetry) => {
                        serde_json::to_string(&telemetry).unwrap()
                    }
                    Event::TelemetryPartialUpdate(telemetry) => {
                        serde_json::to_string(&telemetry).unwrap()
                    }
                    _ => String::from("{}"),
                };

                channel2
                    .basic_publish(
                        "dapnet.telemetry",
                        &*telemetry_routing_key,
                        data.as_bytes(),
                        BasicPublishOptions::default(),
                        BasicProperties::default()
                    )
                    .map(|_| ())
                    .map_err(|_| ())
            }));

            // Consume the messages
            stream.for_each(move |message| {
                use std::str::from_utf8;
                use serde_json;

                let msg: Option<Message> =
                    from_utf8(&message.data).ok().and_then(|str| {
                        serde_json::from_str(&str).ok()
                    });

                if let Some(msg) = msg {
                    event_handler.publish(Event::MessageReceived(msg));
                }
                else {
                    warn!("Could not decode incoming message")
                }
                channel.basic_ack(message.delivery_tag)
            })
        })
        .map_err(|e| {
            telemetry_update!(node: &|node: &mut telemetry::Node| {
            node.connected = false;
            node.connected_since = None;
        });

            warn!("RabbitMQ connection lost. {:?}", e)
        });

    rt.spawn(connection);
}
