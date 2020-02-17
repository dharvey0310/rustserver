use rustserver::ThreadPool;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

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

fn main() {
    // Read in the YAML config file
    let config_file = fs::read_to_string("config.yaml").unwrap();
    let config: Config = serde_yaml::from_str(&config_file).unwrap();

    println!("{}", config.locations[0].path);


    let listener = TcpListener::bind(format!("{}:{}", config.server.listen_address, config.server.port)).unwrap();
    let pool = ThreadPool::new(config.server.thread_count);

    println!("Server listening on port {}", config.server.port);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}