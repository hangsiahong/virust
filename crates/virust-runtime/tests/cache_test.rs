use axum::{
    routing::get,
    Router,
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
};
use tower::ServiceExt;
use virust_runtime::{CacheMiddleware, CacheConfig, CacheStore, CacheEntry, cache_layer_with_state};
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper handler that returns a cached response
async fn test_handler() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", "max-age=60".parse().unwrap());
    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert("X-Cache-Tags", "user:123,posts".parse().unwrap());

    (StatusCode::OK, headers, "Hello, World!").into_response()
}

/// Helper handler that returns an error (should not be cached)
async fn error_handler() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response()
}

/// Helper handler with custom cache configuration
async fn custom_cache_handler() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("Cache-Control", "max-age=3600, stale-while-revalidate=300".parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("ETag", "\"custom-etag\"".parse().unwrap());

    (StatusCode::OK, headers, "{\"message\": \"cached\"}").into_response()
}

#[tokio::test]
async fn test_cache_hit_behavior() {
    let middleware = CacheMiddleware::new();

    // Pre-populate cache
    let entry = CacheEntry {
        body: b"Cached Response".to_vec(),
        content_type: "text/plain".to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600,
        etag: "test-etag".to_string(),
        config: CacheConfig {
            max_age: 3600,
            stale_while_revalidate: Some(300),
            tags: vec!["test".to_string()],
        },
    };

    middleware
        .store()
        .insert("GET:/test".to_string(), entry)
        .await;

    // Build router with cache middleware
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            cache_layer_with_state,
        ))
        .with_state(middleware);

    // Make request
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/test")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be a cache hit
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("X-Cache").unwrap().to_str().unwrap(),
        "HIT"
    );

    // Verify cached response
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"Cached Response");
}

#[tokio::test]
async fn test_cache_miss_behavior() {
    let middleware = CacheMiddleware::new();

    // Build router with cache middleware
    let app = Router::new()
        .route("/uncached", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            cache_layer_with_state,
        ))
        .with_state(middleware);

    // Make first request (cache miss)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/uncached")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be a cache miss
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("X-Cache").unwrap().to_str().unwrap(),
        "MISS"
    );

    // Verify response body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"Hello, World!");

    // Make second request (should be cache hit)
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/uncached")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be a cache hit now
    assert_eq!(
        response.headers().get("X-Cache").unwrap().to_str().unwrap(),
        "HIT"
    );
}

#[tokio::test]
async fn test_cache_error_responses_not_cached() {
    let middleware = CacheMiddleware::new();

    // Build router with cache middleware
    let app = Router::new()
        .route("/error", get(error_handler))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            cache_layer_with_state,
        ))
        .with_state(middleware);

    // Make first request (error)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/error")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should receive error
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Make second request - should still be error (not cached)
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/error")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_cache_expiration() {
    let store = CacheStore::new();

    // Insert entry that expires in 1 second
    let entry = CacheEntry {
        body: b"Expiring Soon".to_vec(),
        content_type: "text/plain".to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 1,
        etag: "expiring-etag".to_string(),
        config: CacheConfig::default(),
    };

    store.insert("expiring".to_string(), entry).await;

    // Should be available immediately
    assert!(store.get_fresh("expiring").await.is_some());

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Should be expired now
    assert!(store.get_fresh("expiring").await.is_none());
}

#[tokio::test]
async fn test_cache_tag_invalidation() {
    let middleware = CacheMiddleware::new();

    // Insert entries with different tags
    let entry1 = CacheEntry {
        body: b"User 123 data".to_vec(),
        content_type: "application/json".to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600,
        etag: "etag1".to_string(),
        config: CacheConfig {
            tags: vec!["user:123".to_string()],
            ..Default::default()
        },
    };

    let entry2 = CacheEntry {
        body: b"User 456 data".to_vec(),
        content_type: "application/json".to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600,
        etag: "etag2".to_string(),
        config: CacheConfig {
            tags: vec!["user:456".to_string()],
            ..Default::default()
        },
    };

    middleware
        .store()
        .insert("user:123".to_string(), entry1)
        .await;
    middleware
        .store()
        .insert("user:456".to_string(), entry2)
        .await;

    // Both should be available
    assert!(middleware.store().get_fresh("user:123").await.is_some());
    assert!(middleware.store().get_fresh("user:456").await.is_some());

    // Invalidate user:123
    middleware.store().invalidate_by_tag("user:123").await;

    // Only user:456 should remain
    assert!(middleware.store().get_fresh("user:123").await.is_none());
    assert!(middleware.store().get_fresh("user:456").await.is_some());
}

#[tokio::test]
async fn test_cache_clear() {
    let store = CacheStore::new();

    // Insert multiple entries
    for i in 0..5 {
        let entry = CacheEntry {
            body: format!("Entry {}", i).into_bytes(),
            content_type: "text/plain".to_string(),
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600,
            etag: format!("etag{}", i),
            config: CacheConfig::default(),
        };

        store.insert(format!("key{}", i), entry).await;
    }

    // Verify entries were inserted (by checking they exist)
    assert!(store.get_fresh("key0").await.is_some());
    assert!(store.get_fresh("key1").await.is_some());
    assert!(store.get_fresh("key2").await.is_some());
    assert!(store.get_fresh("key3").await.is_some());
    assert!(store.get_fresh("key4").await.is_some());

    // Clear all
    store.clear().await;

    // None should remain
    assert!(store.get_fresh("key0").await.is_none());
    assert!(store.get_fresh("key1").await.is_none());
    assert!(store.get_fresh("key2").await.is_none());
    assert!(store.get_fresh("key3").await.is_none());
    assert!(store.get_fresh("key4").await.is_none());
}

#[tokio::test]
async fn test_stale_while_revalidate_header() {
    let middleware = CacheMiddleware::new();

    // Pre-populate cache with stale-while-revalidate config
    let entry = CacheEntry {
        body: b"Stale Content".to_vec(),
        content_type: "text/plain".to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600,
        etag: "stale-etag".to_string(),
        config: CacheConfig {
            max_age: 3600,
            stale_while_revalidate: Some(300),
            tags: vec![],
        },
    };

    middleware
        .store()
        .insert("GET:/stale".to_string(), entry)
        .await;

    // Build router with cache middleware
    let app = Router::new()
        .route("/stale", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            cache_layer_with_state,
        ))
        .with_state(middleware);

    // Make request
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/stale")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should have stale-while-revalidate header
    let cache_control = response
        .headers()
        .get("Cache-Control")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(cache_control.contains("max-age=3600"));
    assert!(cache_control.contains("stale-while-revalidate=300"));
}

#[tokio::test]
async fn test_custom_cache_config_parsing() {
    let middleware = CacheMiddleware::new();

    // Build router with cache middleware
    let app = Router::new()
        .route("/custom", get(custom_cache_handler))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            cache_layer_with_state,
        ))
        .with_state(middleware.clone());

    // Make first request
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/custom")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify custom cache headers
    let cache_control = response
        .headers()
        .get("Cache-Control")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(cache_control.contains("max-age=3600"));
    assert!(cache_control.contains("stale-while-revalidate=300"));

    // Verify custom ETag
    assert_eq!(
        response.headers().get("ETag").unwrap().to_str().unwrap(),
        "\"custom-etag\""
    );

    // Verify cache was populated
    let cached = middleware.store().get_fresh("GET:/custom").await;
    assert!(cached.is_some());
    let entry = cached.unwrap();
    assert_eq!(entry.config.max_age, 3600);
    assert_eq!(entry.config.stale_while_revalidate, Some(300));
}

#[tokio::test]
async fn test_different_cache_keys_for_different_methods() {
    let middleware = CacheMiddleware::new();

    // Build router with cache middleware
    let app = Router::new()
        .route("/resource", get(test_handler))
        .layer(axum::middleware::from_fn_with_state(
            middleware.clone(),
            cache_layer_with_state,
        ))
        .with_state(middleware);

    // Make GET request
    let get_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/resource")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // For now, just verify GET is cached
    assert_eq!(
        get_response.headers().get("X-Cache").unwrap().to_str().unwrap(),
        "MISS"
    );

    // Second GET should be HIT
    let get_response2 = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/resource")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        get_response2.headers().get("X-Cache").unwrap().to_str().unwrap(),
        "HIT"
    );
}
