use std::io::{self, Result};
use std::net::ToSocketAddrs;

use tokio;
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite};
use futures::future::Future;
use futures::{self, IntoFuture, Stream};
use lapin::types::FieldTable;
use lapin::client::{Client, ConnectionOptions};
use lapin::channel::{BasicConsumeOptions, BasicProperties, BasicPublishOptions, ConfirmSelectOptions, QueueBindOptions, QueueDeclareOptions};

use config::Config;
use pocsag::{Message, MessageType};
use pocsag::{Scheduler, TimeSlots};

pub fn consumer(config: &Config, scheduler: Scheduler)
                -> impl Future<Item = (), Error = ()> {
    let call = config.master.call.to_owned().to_ascii_lowercase();
    let user = format!("tx-{}", &call).to_owned();
    let routing_key = format!("{}.*", &call).to_owned();
    let auth_key = config.master.auth.to_owned();
    let host = &config.master.server;
    let port = 5672;

    info!("Connecting to {}:{}...", host, port);

    let addr = (&**host, port).to_socket_addrs().unwrap()
        .next().expect("Cannot resolve hostname");

    TcpStream::connect(&addr).and_then(|stream| {
       Client::connect(stream, ConnectionOptions {
            username: user,
            password: auth_key,
            vhost: "/".to_owned(),
            frame_max: 0,
            heartbeat: 30
        })
    }).and_then(|(client, heartbeat)| {
       tokio::spawn(heartbeat.map_err(|_| ()));
       client.create_channel()
    }).and_then(move |channel| {
        // Declare queue
        channel.queue_declare(&call, QueueDeclareOptions::default(), FieldTable::new())
            .map(|queue| (channel, queue))
    }).and_then(move |(channel, queue)| {
        // Bind queue to exchange
        channel.queue_bind(&queue.name(), "dapnet.transmitters", &*routing_key,
                           QueueBindOptions::default(), FieldTable::new())
            .map(|_| (channel, queue))
    }).and_then(|(channel, queue)| {
        // Create a consumer
        channel.basic_consume(&queue, "consumer", BasicConsumeOptions::default(), FieldTable::new())
            .map(move |stream| (channel, stream))
    }).and_then(|(channel, stream)| {
        info!("Listening for incoming calls.");
        // Consume the messages
        stream.for_each(move |message| {
            use std::str::from_utf8;
            use serde_json;
            match message.routing_key.split(".").nth(1) {
                Some("call") => {
                    let message: Option<Message> = from_utf8(&message.data).ok()
                        .and_then(|str| serde_json::from_str(&str).ok());

                    if let Some(msg) = message {
                        scheduler.message(msg);
                    }
                    else {
                        warn!("Could not decode incoming message")
                    }
                },
                Some(mtype) => {
                    warn!("Received unknown message type {}", mtype);
                }
                None => {
                    warn!("Received unknown message");
                }
            }
            channel.basic_ack(message.delivery_tag)
        })
    }).map_err(|e| (
        warn!("RabbitMQ connection lost. {:?}", e)
    ))
}
