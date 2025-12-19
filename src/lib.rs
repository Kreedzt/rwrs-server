use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapsConfig {
    pub maps: Vec<MapEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEntry {
    pub name: String,
    pub path: String,
    pub image: String,
}

impl MapsConfig {
    pub fn new() -> Self {
        Self {
            maps: Vec::new(),
        }
    }

    pub async fn load_from_file(file_path: &str) -> Result<Self, String> {
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                error!("Failed to get current directory: {}", e);
                return Err(format!("Failed to get current directory: {}", e));
            }
        };

        let full_path = if std::path::Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else {
            current_dir.join(file_path).to_string_lossy().to_string()
        };

        info!("Loading maps configuration from: {} (full path: {})", file_path, full_path);

        let content = match tokio::fs::read_to_string(&full_path).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read maps config file '{}': {}", full_path, e);
                return Err(format!("Failed to read maps config file '{}': {}", full_path, e));
            }
        };

        let config: MapsConfig = match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse maps config file '{}': {}", full_path, e);
                return Err(format!("Failed to parse maps config file '{}': {}", full_path, e));
            }
        };

        info!("Successfully loaded {} map entries from config file", config.maps.len());
        Ok(config)
    }

    pub fn get_maps(&self) -> Vec<MapEntry> {
        self.maps.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub android: RepoVersion,
    pub web: RepoVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoVersion {
    pub version: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
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

pub struct Config {
    pub port: String,
    pub host: String,
    pub cache_duration_secs: u64,
    pub rate_limit_duration_secs: u64,
    pub maps_config_path: String,
    pub android_repo_url: Option<String>,
    pub web_repo_url: Option<String>,
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

        // Maps config file path, default "maps.json"
        let maps_config_path = env::var("MAPS_CONFIG")
            .unwrap_or_else(|_| "maps.json".to_string());

        // Repository URLs for version info
        let android_repo_url = env::var("ANDROID_REPO_URL").ok();
        let web_repo_url = env::var("WEB_REPO_URL").ok();

        Ok(Config {
            port: port.to_string(),
            host: host.to_string(),
            cache_duration_secs,
            rate_limit_duration_secs,
            maps_config_path,
            android_repo_url,
            web_repo_url,
        })
    }
}

pub struct ApiCache {
    cache: Arc<RwLock<Option<CachedResponse>>>,
    last_request_time: Arc<RwLock<Instant>>,
    rate_limit_duration: Duration,
    cache_expiry_duration: Duration,
}

impl ApiCache {
    pub fn new(rate_limit_secs: u64, cache_expiry_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            last_request_time: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(rate_limit_secs + 1))),
            rate_limit_duration: Duration::from_secs(rate_limit_secs),
            cache_expiry_duration: Duration::from_secs(cache_expiry_secs),
        }
    }

    pub async fn get_cached_response(&self, url: &str) -> Result<(String, u16), String> {
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

pub async fn get_latest_tag(repo_url: &str) -> Option<(String, String)> {
    // Extract owner and repo from GitHub URL
    let url_parts: Vec<&str> = repo_url.trim_end_matches('/').split('/').collect();
    if url_parts.len() < 5 || url_parts[2] != "github.com" {
        return None;
    }

    let owner = url_parts[3];
    let repo = url_parts[4];

    // Use GitHub API to get the latest release
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

    let client = reqwest::Client::builder()
        .user_agent("rwrs-server")
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    match client.get(&api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(release_info) = response.json::<serde_json::Value>().await {
                    let tag_name = release_info.get("tag_name")?.as_str()?.to_string();
                    // Return URL to the release page instead of zipball
                    let release_url = format!("https://github.com/{}/{}/releases/tag/{}", owner, repo, tag_name);
                    return Some((tag_name, release_url));
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch release info from {}: {}", api_url, e);
        }
    }

    None
}

#[cfg(test)]
mod tests;