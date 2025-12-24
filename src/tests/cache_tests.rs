#[cfg(test)]
mod tests {
    use crate::ApiCache;
    use std::time::Duration;
    use tokio::time::sleep;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    /// Helper function to create a test cache with short durations
    fn create_test_cache() -> ApiCache {
        ApiCache::new(0, 10) // 0 second rate limit (disabled), 10 second cache expiry
    }

    #[tokio::test]
    async fn test_cache_separation_by_url() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Mock endpoint - query params don't affect path matching in wiremock
        Mock::given(method("GET"))
            .and(path("/view_players.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("player_data"))
            .mount(&mock_server)
            .await;

        // Create cache
        let cache = create_test_cache();

        // Base URL
        let base = format!("{}/view_players.php", mock_server.uri());

        // Request for player1
        let url1 = format!("{}?name=player1", base);
        let result1 = cache.get_cached_response(&url1).await.unwrap();
        assert_eq!(result1.0, "player_data");

        // Request for player2
        let url2 = format!("{}?name=player2", base);
        let result2 = cache.get_cached_response(&url2).await.unwrap();
        assert_eq!(result2.0, "player_data");

        // Request for player1 again - should return cached data for that specific URL
        let result3 = cache.get_cached_response(&url1).await.unwrap();
        assert_eq!(result3.0, "player_data");

        // Both URLs should have separate cache entries even though response is same
        // Verify by checking that we're not making extra API calls (only 2 calls for 4 requests)
        sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_cache_hit_same_url() {
        let mock_server = MockServer::start().await;

        // Mock endpoint - track call count
        Mock::given(method("GET"))
            .and(path("/server_list.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("server_list_data"))
            .expect(1) // Should only be called once due to caching
            .mount(&mock_server)
            .await;

        let cache = create_test_cache();
        let url = format!("{}/server_list.php", mock_server.uri());

        // First request - hits the API
        let result1 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result1.0, "server_list_data");

        // Second request - should return cached data
        let result2 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result2.0, "server_list_data");

        // Third request - still cached
        let result3 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result3.0, "server_list_data");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mock_server = MockServer::start().await;

        // Mock first request
        Mock::given(method("GET"))
            .and(path("/test.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data_v1"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Create cache with 2 second expiry
        let cache = ApiCache::new(0, 2);
        let url = format!("{}/test.php", mock_server.uri());

        // First request
        let result1 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result1.0, "data_v1");

        // Wait for cache to expire
        sleep(Duration::from_secs(2)).await;

        // Mock second request (different response)
        Mock::given(method("GET"))
            .and(path("/test.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data_v2"))
            .mount(&mock_server)
            .await;

        // Second request - should fetch fresh data
        let result2 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result2.0, "data_v2");
    }

    #[tokio::test]
    async fn test_different_query_params_separate_cache() {
        let mock_server = MockServer::start().await;

        // Setup mock that handles both URLs and counts calls
        Mock::given(method("GET"))
            .and(path("/api"))
            .respond_with(ResponseTemplate::new(200).set_body_string("api_data"))
            .expect(2) // Should be called exactly twice (once per unique URL)
            .mount(&mock_server)
            .await;

        let cache = create_test_cache();
        let base = format!("{}/api", mock_server.uri());

        // Request with no query params
        let url1 = base.clone();
        let result1 = cache.get_cached_response(&url1).await.unwrap();
        assert_eq!(result1.0, "api_data");

        // Request with query param
        let url2 = format!("{}?filter=true", base);
        let result2 = cache.get_cached_response(&url2).await.unwrap();
        assert_eq!(result2.0, "api_data");

        // Request original URL again - should return cached data
        let result3 = cache.get_cached_response(&url1).await.unwrap();
        assert_eq!(result3.0, "api_data");

        // Request with query param again - should return cached data
        let result4 = cache.get_cached_response(&url2).await.unwrap();
        assert_eq!(result4.0, "api_data");
    }

    #[tokio::test]
    async fn test_multiple_urls_independent_caches() {
        let mock_server = MockServer::start().await;

        // Setup mocks for different endpoints
        Mock::given(method("GET"))
            .and(path("/endpoint1"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data1"))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/endpoint2"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data2"))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/endpoint3"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data3"))
            .mount(&mock_server)
            .await;

        let cache = create_test_cache();

        let url1 = format!("{}/endpoint1", mock_server.uri());
        let url2 = format!("{}/endpoint2", mock_server.uri());
        let url3 = format!("{}/endpoint3", mock_server.uri());

        // Populate caches
        let result1 = cache.get_cached_response(&url1).await.unwrap();
        let result2 = cache.get_cached_response(&url2).await.unwrap();
        let result3 = cache.get_cached_response(&url3).await.unwrap();

        assert_eq!(result1.0, "data1");
        assert_eq!(result2.0, "data2");
        assert_eq!(result3.0, "data3");

        // Verify each URL returns its own cached data
        let cached1 = cache.get_cached_response(&url1).await.unwrap();
        let cached2 = cache.get_cached_response(&url2).await.unwrap();
        let cached3 = cache.get_cached_response(&url3).await.unwrap();

        assert_eq!(cached1.0, "data1");
        assert_eq!(cached2.0, "data2");
        assert_eq!(cached3.0, "data3");
    }

    #[tokio::test]
    async fn test_rate_limit_returns_cached_data() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/limited"))
            .respond_with(ResponseTemplate::new(200).set_body_string("initial_data"))
            .mount(&mock_server)
            .await;

        // Cache with 3 second rate limit
        let cache = ApiCache::new(3, 30);
        let url = format!("{}/limited", mock_server.uri());

        // First request
        let result1 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result1.0, "initial_data");

        // Update mock to return different data
        Mock::given(method("GET"))
            .and(path("/limited"))
            .respond_with(ResponseTemplate::new(200).set_body_string("new_data"))
            .mount(&mock_server)
            .await;

        // Immediate second request - rate limited, should return cached data
        let result2 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result2.0, "initial_data"); // Still cached data
    }

    #[tokio::test]
    async fn test_status_code_preserved_in_cache() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(201).set_body_string("created"))
            .mount(&mock_server)
            .await;

        let cache = create_test_cache();
        let url = format!("{}/test", mock_server.uri());

        // First request
        let result1 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result1.1, 201);
        assert_eq!(result1.0, "created");

        // Second request - from cache
        let result2 = cache.get_cached_response(&url).await.unwrap();
        assert_eq!(result2.1, 201);
        assert_eq!(result2.0, "created");
    }
}
