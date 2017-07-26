use tiny_http;

pub fn run() {
    let server = tiny_http::Server::http("0.0.0.0:8073").unwrap();
    while let Ok(req) = server.recv() {
        let (mime, data) = match req.url() {
            "/main.js" => ("application/javascript", include_bytes!("assets/main.js").to_vec()),
            "/vue.js" => ("application/javascript", include_bytes!("assets/vue.js").to_vec()),
            "/style.css" => ("text/css", include_bytes!("assets/style.css").to_vec()),
            "/logo.png" => ("image/png", include_bytes!("assets/logo.png").to_vec()),
            "/pin_numbers.png" => ("image/png", include_bytes!("assets/pin_numbers.png").to_vec()),
            _ => ("text/html", include_bytes!("assets/index.html").to_vec())
        };

        let res = tiny_http::Response::from_data(data).with_header(
            tiny_http::Header {
                field: "Content-Type".parse().unwrap(),
                value: mime.parse().unwrap()
            }
        );

        req.respond(res).ok();
    }
}
