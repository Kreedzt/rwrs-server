use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use salvo::affix_state;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
struct CachedResponse {
    data: String,
    timestamp: Instant,
    status_code: u16,
}

impl CachedResponse {
    fn new(data: String, status_code: u16) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            status_code,
        }
    }

    fn is_expired(&self, duration: Duration) -> bool {
        self.timestamp.elapsed() > duration
    }
}

struct Config {
    port: String,
    host: String,
    cache_duration_secs: u64,
    rate_limit_duration_secs: u64,
}

impl Config {
    pub fn new() -> Result<Config, &'static str> {
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "5800".to_string());

        // Cache expiry time in seconds, default 3
        let cache_duration_secs = env::var("CACHE_DURATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        // Rate limit time in seconds, default 3
        let rate_limit_duration_secs = env::var("RATE_LIMIT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        Ok(Config {
            port: port.to_string(),
            host: host.to_string(),
            cache_duration_secs,
            rate_limit_duration_secs,
        })
    }
}

struct ServerListCache {
    cache: Arc<RwLock<Option<CachedResponse>>>,
    last_request_time: Arc<RwLock<Instant>>,
    rate_limit_duration: Duration,
    cache_expiry_duration: Duration,
}

impl ServerListCache {
    fn new(rate_limit_secs: u64, cache_expiry_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            last_request_time: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(rate_limit_secs + 1))),
            rate_limit_duration: Duration::from_secs(rate_limit_secs),
            cache_expiry_duration: Duration::from_secs(cache_expiry_secs),
        }
    }

    async fn get_server_list(&self, url: &str) -> Result<(String, u16), String> {
        let now = Instant::now();

        // Check cache
        {
            let cache_guard = self.cache.read().await;
            if let Some(cached) = cache_guard.as_ref() {
                if !cached.is_expired(self.cache_expiry_duration) {
                    info!("Cache hit, age: {:?}", cached.timestamp.elapsed());
                    return Ok((cached.data.clone(), cached.status_code));
                } else {
                    info!("Cache expired, refreshing required");
                }
            } else {
                info!("No cache data available, fetching from API");
            }
        }

        // Check rate limiting
        {
            let last_request_guard = self.last_request_time.read().await;
            if now.duration_since(*last_request_guard) < self.rate_limit_duration {
                warn!("Rate limit exceeded, returning cached data");
                let cache_guard = self.cache.read().await;
                if let Some(cached) = cache_guard.as_ref() {
                    return Ok((cached.data.clone(), cached.status_code));
                } else {
                    return Err("Rate limit exceeded and no cache available".to_string());
                }
            }
        }

        // Update last request time
        {
            let mut last_request_guard = self.last_request_time.write().await;
            *last_request_guard = now;
        }

        // Make actual API call
        info!("Making request to external API: {}", url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                info!("Received response with status: {}", status);
                match response.text().await {
                    Ok(text) => {
                        info!("Successfully fetched {} bytes from API", text.len());
                        // Cache the response
                        self.update_cache(text.clone(), status).await;
                        Ok((text, status))
                    }
                    Err(e) => {
                        error!("Failed to read response body: {}", e);
                        Err(format!("Failed to read response body: {}", e))
                    }
                }
            }
            Err(e) => {
                error!("Request failed: {}", e);
                Err(format!("Request failed: {}", e))
            }
        }
    }

    async fn update_cache(&self, data: String, status_code: u16) {
        let cached_response = CachedResponse::new(data, status_code);
        let mut cache_guard = self.cache.write().await;
        *cache_guard = Some(cached_response);
    }
}

#[handler]
async fn ping() -> &'static str {
    "pong"
}

#[handler]
async fn proxy_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // Get cache from depot
    let cache = depot.obtain::<Arc<ServerListCache>>().unwrap();

    // Get query string from original request
    let query_string = req.uri().query().unwrap_or("");
    let base_url = "http://rwr.runningwithrifles.com/rwr_server_list/get_server_list.php";
    let url = if query_string.is_empty() {
        base_url.to_string()
    } else {
        format!("{}?{}", base_url, query_string)
    };

    match cache.get_server_list(&url).await {
        Ok((data, status_code)) => {
            // Cache successful response
            res.status_code(StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK));
            res.render(Text::Html(data));
        }
        Err(e) => {
            error!("Failed to get server list: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain(format!("Unable to fetch server list: {}", e)));
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let c = Config::new().unwrap();
    let listen_addr = format!("{}:{}", c.host, c.port);
    info!("listening at {}", listen_addr);

    // Create cache instance
    let cache = Arc::new(ServerListCache::new(c.rate_limit_duration_secs, c.cache_duration_secs));

    info!("Cache and rate limiting mechanism enabled:");
    info!("  - Rate limit interval: {} seconds", c.rate_limit_duration_secs);
    info!("  - Cache expiry time: {} seconds", c.cache_duration_secs);

    let router = Router::new()
        .hoop(RequestId::new())
        .push(Router::new().path("/ping").get(ping))
        .push(Router::new()
            .path("/api/server_list")
            .hoop(affix_state::inject(cache.clone()))
            .goal(proxy_handler)
        )
        .push(Router::with_path("{**path}")
            .get(StaticDir::new(["static"]).defaults("index.html")));

    let service = Service::new(router).hoop(Logger::new());
    let acceptor = TcpListener::new(listen_addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
