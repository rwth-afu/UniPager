use std::net::SocketAddr;

use bytes::buf::Buf;

use serde_json;
use tokio::runtime::Runtime;

use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use crate::event::{Event, EventHandler};
use crate::telemetry;

fn file_response(data: &[u8], content_type: &str) -> Response<Body> {
    let body = Body::from(data.to_vec());
    Response::builder()
        .header("Content-type", content_type)
        .body(body)
        .unwrap()
}

async fn response(req: Request<Body>, event_handler: EventHandler) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path())
    {
        (&Method::GET, "/") |
        (&Method::GET, "/index.html") => {
            Ok(file_response(
                include_bytes!("assets/index.html"),
                "text/html"
            ))
        }
        (&Method::GET, "/main.js") => {
            Ok(file_response(
                include_bytes!("assets/main.js"),
                "application/javascript"
            ))
        }
        (&Method::GET, "/vue.js") => {
            Ok(file_response(
                include_bytes!("assets/vue.js"),
                "application/javascript"
            ))
        }
        (&Method::GET, "/style.css") => {
            Ok(file_response(
                include_bytes!("assets/style.css"),
                "text/css"
            ))
        }
        (&Method::GET, "/logo.png") => {Ok(file_response(
                include_bytes!("assets/logo.png"),
                "image/png"
            ))
        }
        (&Method::GET, "/pin_numbers.png") => {
            Ok(file_response(
                include_bytes!("assets/pin_numbers.png"),
                "image/png"
            ))
        }
        (&Method::GET, "/telemetry") => {
            let telemetry = serde_json::to_string(&telemetry::get());
            let data = telemetry.unwrap().as_bytes().to_vec();
            let body = Body::from(data);

            Ok(
                Response::builder()
                    .header("Content-type", "application/json")
                    .body(body)
                    .unwrap()
            )
        }
        (&Method::POST, "/message") => {
            let body = hyper::body::to_bytes(req).await.unwrap();

            if let Ok(msg) = serde_json::from_slice(&body) {
                event_handler.publish(Event::MessageReceived(msg));
                let body = Body::from("{\"ok\": true}");
                Ok(Response::builder().body(body).unwrap())
            }
            else {
                let body = Body::from("{\"ok\": false}");
                Ok(Response::builder().body(body).unwrap())
            }
        }
        _ => {
            let body = Body::from("Not found");
            Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap()
            )
        }
    }
}

pub fn start(runtime: &Runtime, event_handler: EventHandler) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8073));

    let new_service = make_service_fn(move |_| {
        let event_handler2 = event_handler.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let event_handler3 = event_handler2.clone();
                response(req, event_handler3)
            }))
        }
    });

    runtime.spawn(async move {
        Server::bind(&addr).serve(new_service).await;
    });
}
