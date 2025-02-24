use salvo::prelude::*;
use tracing::{info};

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new().hoop(
        RequestId::new()
    ).get(hello);
    let service = Service::new(router).hoop(Logger::new());
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(service).await;
}
