use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use salvo::affix_state;
use std::sync::Arc;
use tracing::{error, info};

// Import from lib.rs
use rwrs_server::{
    Config, MapsConfig, ApiCache, VersionInfo, RepoVersion,
    get_latest_tag
};

#[handler]
async fn ping() -> &'static str {
    "pong"
}

#[handler]
async fn maps_handler(depot: &mut Depot, res: &mut Response) {
    let maps_config = depot.obtain::<Arc<MapsConfig>>().unwrap();
    let maps = maps_config.get_maps();

    res.render(Json(&maps));
}

#[handler]
async fn version_handler(depot: &mut Depot, res: &mut Response) {
    let config = depot.obtain::<Arc<Config>>().unwrap();

    let android_version = if let Some(ref android_repo_url) = config.android_repo_url {
        match get_latest_tag(android_repo_url).await {
            Some((version, url)) => RepoVersion {
                version: Some(version),
                url: Some(url),
            },
            None => RepoVersion {
                version: None,
                url: None,
            },
        }
    } else {
        RepoVersion {
            version: None,
            url: None,
        }
    };

    let web_version = if let Some(ref web_repo_url) = config.web_repo_url {
        match get_latest_tag(web_repo_url).await {
            Some((version, url)) => RepoVersion {
                version: Some(version),
                url: Some(url),
            },
            None => RepoVersion {
                version: None,
                url: None,
            },
        }
    } else {
        RepoVersion {
            version: None,
            url: None,
        }
    };

    let version_info = VersionInfo {
        android: android_version,
        web: web_version,
    };

    res.render(Json(&version_info));
}

#[handler]
async fn servers_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // Get cache from depot
    let cache = depot.obtain::<Arc<ApiCache>>().unwrap();

    // Get query string from original request
    let query_string = req.uri().query().unwrap_or("");
    let base_url = "http://rwr.runningwithrifles.com/rwr_server_list/get_server_list.php";
    let url = if query_string.is_empty() {
        base_url.to_string()
    } else {
        format!("{}?{}", base_url, query_string)
    };

    match cache.get_cached_response(&url).await {
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

#[handler]
async fn players_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // Get cache from depot
    let cache = depot.obtain::<Arc<ApiCache>>().unwrap();

    // Get query string from original request
    let query_string = req.uri().query().unwrap_or("");
    let base_url = "http://rwr.runningwithrifles.com/rwr_stats/view_players.php";
    let url = if query_string.is_empty() {
        base_url.to_string()
    } else {
        format!("{}?{}", base_url, query_string)
    };

    match cache.get_cached_response(&url).await {
        Ok((data, status_code)) => {
            // Cache successful response
            res.status_code(StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK));
            res.render(Text::Html(data));
        }
        Err(e) => {
            error!("Failed to get players data: {}", e);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain(format!("Unable to fetch players data: {}", e)));
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let config = Config::new().unwrap();
    let listen_addr = format!("{}:{}", config.host, config.port);
    info!("listening at {}", listen_addr);

    // Create cache instance
    let cache = Arc::new(ApiCache::new(config.rate_limit_duration_secs, config.cache_duration_secs));

    // Load maps configuration
    info!("Loading maps configuration from: {}", config.maps_config_path);
    let maps_config = match MapsConfig::load_from_file(&config.maps_config_path).await {
        Ok(maps) => Arc::new(maps),
        Err(e) => {
            error!("Failed to load maps configuration: {}. Using empty configuration.", e);
            Arc::new(MapsConfig::new())
        }
    };

    info!("Cache and rate limiting mechanism enabled:");
    info!("  - Rate limit interval: {} seconds", config.rate_limit_duration_secs);
    info!("  - Cache expiry time: {} seconds", config.cache_duration_secs);
    info!("  - Maps config file: {}", config.maps_config_path);
    if let Some(ref url) = config.android_repo_url {
        info!("  - Android repo URL: {}", url);
    }
    if let Some(ref url) = config.web_repo_url {
        info!("  - Web repo URL: {}", url);
    }

    // Create config for sharing
    let config = Arc::new(config);

    let router = Router::new()
        .hoop(RequestId::new())
        .push(Router::new().path("/ping").get(ping))
        .push(Router::new().path("/api/version").hoop(affix_state::inject(config.clone())).get(version_handler))
        .push(Router::new().path("/api/maps").hoop(affix_state::inject(maps_config.clone())).get(maps_handler))
        .push(Router::new()
            .path("/api/server_list")
            .hoop(affix_state::inject(cache.clone()))
            .goal(servers_handler)
        )
        .push(Router::new()
            .path("/api/player_list")
            .hoop(affix_state::inject(cache.clone()))
            .goal(players_handler)
        )
        .push(Router::with_path("{**path}")
            .get(StaticDir::new(["static"]).defaults("index.html")));

    let service = Service::new(router).hoop(Logger::new());
    let acceptor = TcpListener::new(listen_addr).bind().await;
    Server::new(acceptor).serve(service).await;
}