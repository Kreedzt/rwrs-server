#[cfg(test)]
mod tests {
    use crate::{Announcement, Config, MapEntry, MapsConfig, RepoVersion, VersionInfo};

    #[tokio::test]
    async fn test_config_default_values() {
        // Remove environment variables to test defaults
        unsafe {
            std::env::remove_var("HOST");
            std::env::remove_var("PORT");
            std::env::remove_var("CACHE_DURATION_SECS");
            std::env::remove_var("MAPS_CONFIG");
            std::env::remove_var("ANDROID_REPO_URL");
            std::env::remove_var("WEB_REPO_URL");
            std::env::remove_var("ANNOUNCEMENT_PATH");
        }

        let config = Config::new().unwrap();

        // Test default values
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, "5800");
        assert_eq!(config.cache_duration_secs, 3);
        assert_eq!(config.maps_config_path, "maps.json");
        assert!(config.android_repo_url.is_none());
        assert!(config.web_repo_url.is_none());
        assert!(config.announcement_path.is_none());
    }

    #[tokio::test]
    async fn test_announcement_disabled_default() {
        let announcement = Announcement::disabled();
        assert!(!announcement.enabled);
        assert_eq!(announcement.html, "");

        // Disabled announcements still serialize to the frontend contract.
        let json = serde_json::to_string(&announcement).unwrap();
        assert_eq!(json, r#"{"enabled":false,"html":""}"#);
    }

    #[tokio::test]
    async fn test_announcement_missing_file_disabled() {
        let announcement =
            Announcement::load_from_file("this/path/does/not/exist/announcement.html").await;
        assert!(!announcement.enabled);
        assert_eq!(announcement.html, "");
    }

    #[tokio::test]
    async fn test_announcement_loads_file_content() {
        let dir = std::env::temp_dir();
        let path = dir.join("rwrs_test_announcement.html");
        let html = "<div class=\"notice\">Hello</div>";
        tokio::fs::write(&path, html).await.unwrap();

        let announcement = Announcement::load_from_file(path.to_str().unwrap()).await;
        assert!(announcement.enabled);
        assert_eq!(announcement.html, html);

        tokio::fs::remove_file(&path).await.ok();
    }

    #[tokio::test]
    async fn test_announcement_empty_file_disabled() {
        let dir = std::env::temp_dir();
        let path = dir.join("rwrs_test_announcement_empty.html");
        tokio::fs::write(&path, "   \n\t  ").await.unwrap();

        let announcement = Announcement::load_from_file(path.to_str().unwrap()).await;
        assert!(!announcement.enabled);
        assert_eq!(announcement.html, "");

        tokio::fs::remove_file(&path).await.ok();
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
