use std::net::SocketAddr;

use serde_json;
use status;
use futures::Future;
use futures::future::ok;

use hyper::{self, Body, Client, Method, Request, Response, Server, StatusCode};
use hyper::client::HttpConnector;
use hyper::service::service_fn;

fn file_response(data: &[u8], content_type: &str) -> Response<Body> {
    let body = Body::from(data.to_vec());
    Response::builder()
       .header("Content-type", content_type)
       .body(body)
       .unwrap()
}

fn response(req: Request<Body>, client: &Client<HttpConnector>)
                     -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            ok(file_response(include_bytes!("assets/index.html"), "text/html"))
        },
        (&Method::GET, "/main.js") => {
            ok(file_response(include_bytes!("assets/main.js"), "application/javascript"))
        },
        (&Method::GET, "/vue.js") => {
            ok(file_response(include_bytes!("assets/vue.js"), "application/javascript"))
        },
        (&Method::GET, "/style.css") => {
            ok(file_response(include_bytes!("assets/style.css"), "text/css"))
        },
        (&Method::GET, "/logo.png") => {
            ok(file_response(include_bytes!("assets/logo.png"), "image/png"))
        },
        (&Method::GET, "/pin_numbers.png") => {
            ok(file_response(include_bytes!("assets/pin_numbers.png"), "image/png"))
        },
        (&Method::GET, "/status") => {
            let status = serde_json::to_string(&status::get());
            let data = status.unwrap().as_bytes().to_vec();
            let body = Body::from(data);

            ok(Response::builder()
               .header("Content-type", "application/json")
               .body(body)
               .unwrap())
        },
        _ => {
            let body = Body::from("Not found");
            ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body)
                .unwrap())
        }
    }
}

pub fn server() -> impl Future<Item = (), Error = ()> {
    let client = Client::new();

    let new_service = move || {
        let client = client.clone();
        service_fn(move |req| {
            response(req, &client)
        })
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 8073));

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    server
}
