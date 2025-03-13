use salvo::prelude::*;
use salvo::proxy::{Proxy, ReqwestClient};
use salvo::serve_static::StaticDir;
use std::env;
use tracing::info;

struct Config {
    port: String,
    host: String,
}

impl Config {
    pub fn new() -> Result<Config, &'static str> {
        let port = env::var("PORT").unwrap_or(String::from("127.0.0.1"));
        let host = env::var("HOST").unwrap_or(String::from("5800"));

        Ok(Config { port: port.to_string(), host: host.to_string() })
    }
}

#[handler]
async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let c = Config::new().unwrap();

    let listen_addr = format!("{}:{}", c.host, c.port);
    info!("listening at {}", listen_addr);

    let router = Router::new()
        .hoop(RequestId::new())
        .push(Router::new().path("/ping").get(ping))
        .push(Router::new().path("/api/server_list").goal(Proxy::new(
            vec!["http://rwr.runningwithrifles.com/rwr_server_list/get_server_list.php"],
            ReqwestClient::default(),
        )))
        .push(Router::with_path("{**path}").get(StaticDir::new(["static"]).defaults("index.html")));
    let service = Service::new(router).hoop(Logger::new());
    let acceptor = TcpListener::new(listen_addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
