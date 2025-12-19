#[cfg(test)]
mod tests {
    use crate::{Config, MapsConfig, VersionInfo, RepoVersion, MapEntry};

    #[tokio::test]
    async fn test_config_default_values() {
        // Create config without environment variables to test defaults
        let config = Config::new().unwrap();

        // Test default values
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, "5800");
        assert_eq!(config.cache_duration_secs, 3);
        assert_eq!(config.rate_limit_duration_secs, 3);
        assert_eq!(config.maps_config_path, "maps.json");
        assert!(config.android_repo_url.is_none());
        assert!(config.web_repo_url.is_none());
    }

    #[tokio::test]
    async fn test_maps_config_creation() {
        let config = MapsConfig::new();
        assert_eq!(config.get_maps().len(), 0);
    }

    #[tokio::test]
    async fn test_version_info_creation() {
        let version_info = VersionInfo {
            android: RepoVersion {
                version: Some("v2.0.0".to_string()),
                url: Some("https://github.com/example/android/releases/tag/v2.0.0".to_string()),
            },
            web: RepoVersion {
                version: None,
                url: None,
            },
        };

        // Test serialization
        let json = serde_json::to_string(&version_info).unwrap();
        let parsed: VersionInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.android.version, Some("v2.0.0".to_string()));
        assert_eq!(parsed.web.version, None);
    }

    #[tokio::test]
    async fn test_version_info_empty() {
        let version_info = VersionInfo {
            android: RepoVersion {
                version: None,
                url: None,
            },
            web: RepoVersion {
                version: None,
                url: None,
            },
        };

        let json = serde_json::to_string(&version_info).unwrap();
        let parsed: VersionInfo = serde_json::from_str(&json).unwrap();

        assert!(parsed.android.version.is_none());
        assert!(parsed.web.version.is_none());
    }

    #[tokio::test]
    async fn test_map_entry_creation() {
        let entry = MapEntry {
            name: "Test Map".to_string(),
            path: "media/packages/vanilla/maps/test".to_string(),
            image: "test.png".to_string(),
        };

        assert_eq!(entry.name, "Test Map");
        assert_eq!(entry.path, "media/packages/vanilla/maps/test");
        assert_eq!(entry.image, "test.png");
    }
}