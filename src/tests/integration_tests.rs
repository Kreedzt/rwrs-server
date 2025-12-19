#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use std::fs;
    use crate::MapsConfig;

    #[tokio::test]
    async fn test_load_valid_config_file() {
        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_maps.json");

        // Create test maps configuration
        let test_config = r#"
        {
            "maps": [
                {
                    "name": "Test Map 1",
                    "path": "media/packages/vanilla.desert/maps/map1",
                    "image": "map1.png"
                },
                {
                    "name": "Test Map 2",
                    "path": "media/packages/vanilla.jungle/maps/map2",
                    "image": "https://example.com/map2.png"
                }
            ]
        }
        "#;

        // Write the test configuration to file
        fs::write(&file_path, test_config).unwrap();

        // Load the configuration
        let config = MapsConfig::load_from_file(file_path.to_str().unwrap()).await.unwrap();
        let maps = config.get_maps();

        // Verify the loaded maps
        assert_eq!(maps.len(), 2);
        assert_eq!(maps[0].name, "Test Map 1");
        assert_eq!(maps[0].path, "media/packages/vanilla.desert/maps/map1");
        assert_eq!(maps[0].image, "map1.png");
        assert_eq!(maps[1].name, "Test Map 2");
        assert_eq!(maps[1].path, "media/packages/vanilla.jungle/maps/map2");
        assert_eq!(maps[1].image, "https://example.com/map2.png");
    }

    #[tokio::test]
    async fn test_load_empty_config() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty_maps.json");

        let test_config = r#"
        {
            "maps": []
        }
        "#;

        fs::write(&file_path, test_config).unwrap();

        let config = MapsConfig::load_from_file(file_path.to_str().unwrap()).await.unwrap();
        assert_eq!(config.get_maps().len(), 0);
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let result = MapsConfig::load_from_file("nonexistent_file.json").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read maps config file"));
    }
}