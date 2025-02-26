use salvo::prelude::*;
use salvo::proxy::{Proxy, ReqwestClient};
use salvo::serve_static::StaticDir;
use tracing::info;

#[handler]
async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new()
        .hoop(RequestId::new())
        .push(Router::new().path("/ping").get(ping))
        .push(Router::new().path("/api/server_list").goal(Proxy::new(
            vec!["http://rwr.runningwithrifles.com/rwr_server_list/get_server_list.php"],
            ReqwestClient::default(),
        )))
        .push(Router::with_path("{**path}").get(StaticDir::new(["static"]).defaults("index.html")));
    let service = Service::new(router).hoop(Logger::new());
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(service).await;
}
