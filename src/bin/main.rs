use serde::{Serialize, Deserialize};
use futures_util::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server as HyperServer, StatusCode};
use std::fs;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    server: Server,
    locations: Vec<Location>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Server {
    thread_count: usize,
    listen_address: String,
    port: i64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Location {
    path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read in the YAML config file
    let config_file = fs::read_to_string("config.yaml").unwrap();
    let config: Config = serde_yaml::from_str(&config_file).unwrap();

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_connection)) });

    let listen_addr: std::net::SocketAddr = format!("{}:{}", config.server.listen_address, config.server.port).parse().unwrap();

    let web_server = HyperServer::bind(&listen_addr).serve(service);

    let graceful = web_server.with_graceful_shutdown(shutdown_signal());

    println!("Server listening on port {}", config.server.port);

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let contents = fs::read_to_string("hello.html").unwrap();
            Ok(Response::new(Body::from(contents)))
        },

        _ => {
            let contents = fs::read_to_string("404.html").unwrap();
            let mut not_found = Response::new(Body::from(contents));
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("failed to install CTRL+C signal handler");
}