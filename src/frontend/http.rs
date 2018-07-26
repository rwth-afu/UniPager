use std::net::SocketAddr;

use futures::{Future, Stream};
use futures::future::ok;
use serde_json;
use tokio::runtime::Runtime;

use hyper::{self, Body, Method, Request, Response, Server, StatusCode};
use hyper::service::service_fn;

use event::{Event, EventHandler};
use telemetry;

fn file_response(data: &[u8], content_type: &str) -> Response<Body> {
    let body = Body::from(data.to_vec());
    Response::builder()
        .header("Content-type", content_type)
        .body(body)
        .unwrap()
}

fn response(req: Request<Body>, event_handler: EventHandler)
    -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    match (req.method(), req.uri().path())
    {
        (&Method::GET, "/") |
        (&Method::GET, "/index.html") => {
            ok(file_response(
                include_bytes!("assets/index.html"),
                "text/html"
            ))
        }
        (&Method::GET, "/main.js") => {
            ok(file_response(
                include_bytes!("assets/main.js"),
                "application/javascript"
            ))
        }
        (&Method::GET, "/vue.js") => {
            ok(file_response(
                include_bytes!("assets/vue.js"),
                "application/javascript"
            ))
        }
        (&Method::GET, "/style.css") => {
            ok(file_response(
                include_bytes!("assets/style.css"),
                "text/css"
            ))
        }
        (&Method::GET, "/logo.png") => {
            ok(file_response(
                include_bytes!("assets/logo.png"),
                "image/png"
            ))
        }
        (&Method::GET, "/pin_numbers.png") => {
            ok(file_response(
                include_bytes!("assets/pin_numbers.png"),
                "image/png"
            ))
        }
        (&Method::GET, "/telemetry") => {
            let telemetry = serde_json::to_string(&telemetry::get());
            let data = telemetry.unwrap().as_bytes().to_vec();
            let body = Body::from(data);

            ok(
                Response::builder()
                    .header("Content-type", "application/json")
                    .body(body)
                    .unwrap()
            )
        }
        (&Method::POST, "/message") => {
            let res = req.into_body()
                .concat2()
                .and_then(move |body| {
                    if let Ok(msg) = serde_json::from_slice(&body) {
                        event_handler.publish(Event::MessageReceived(msg));
                    };
                    ok(())
                }).wait();

            if res.is_ok() {
                let body = Body::from("{\"ok\": true}");
                ok(Response::builder().body(body).unwrap())
            }
            else {
                let body = Body::from("{\"ok\": false}");
                ok(Response::builder().body(body).unwrap())
            }
        }
        _ => {
            let body = Body::from("Not found");
            ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap()
            )
        }
    }
}

pub fn start(rt: &mut Runtime, event_handler: EventHandler) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8073));

    let server = Server::bind(&addr)
        .serve(move || {
            let event_handler2 = event_handler.clone();
            service_fn(move |req| {
                let event_handler3 = event_handler2.clone();
                response(req, event_handler3)
            })
        })
        .map_err(|e| eprintln!("server error: {}", e));

    rt.spawn(server);
}
