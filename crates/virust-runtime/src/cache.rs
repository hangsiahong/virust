use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum time to cache the response (in seconds)
    pub max_age: u64,

    /// Time to serve stale content while revalidating (in seconds)
    pub stale_while_revalidate: Option<u64>,

    /// Optional tags for cache invalidation
    pub tags: Vec<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_age: 3600, // 1 hour default
            stale_while_revalidate: None,
            tags: Vec::new(),
        }
    }
}

/// A cached response entry
#[derive(Clone)]
pub struct CacheEntry {
    /// Response body
    pub body: Vec<u8>,

    /// Content type header
    pub content_type: String,

    /// Expiration timestamp (Unix timestamp)
    pub expires_at: u64,

    /// ETag for validation
    pub etag: String,

    /// Cache configuration
    pub config: CacheConfig,
}

/// In-memory cache store
#[derive(Clone)]
pub struct CacheStore {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl CacheStore {
    /// Create a new cache store
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a fresh cache entry (not expired)
    pub async fn get_fresh(&self, key: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().await;
        let entry = entries.get(key)?;

        // Check if entry is still fresh
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < entry.expires_at {
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Insert a cache entry
    pub async fn insert(&self, key: String, entry: CacheEntry) {
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
    }

    /// Invalidate cache entries by tag
    pub async fn invalidate_by_tag(&self, tag: &str) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| !entry.config.tags.contains(&tag.to_string()));
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache middleware
#[derive(Clone)]
pub struct CacheMiddleware {
    store: CacheStore,
}

impl CacheMiddleware {
    /// Create a new cache middleware
    pub fn new() -> Self {
        Self {
            store: CacheStore::new(),
        }
    }

    /// Create a new cache middleware with custom store
    pub fn with_store(store: CacheStore) -> Self {
        Self { store }
    }

    /// Get the cache store
    pub fn store(&self) -> &CacheStore {
        &self.store
    }
}

impl Default for CacheMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a cache key from the request
fn generate_cache_key(req: &Request) -> String {
    let uri = req.uri().to_string();
    let method = req.method().to_string();

    // Simple key generation - in production you might want to include
    // query parameters, headers, or body hash
    format!("{}:{}", method, uri)
}

/// Generate ETag from content
fn generate_etag(content: &[u8]) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Build a cached response with appropriate headers
fn build_cached_response(entry: &CacheEntry, is_hit: bool) -> Response {
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", &entry.content_type)
        .header("Cache-Control", format!("max-age={}", entry.config.max_age))
        .header("ETag", &entry.etag)
        .body(axum::body::Body::from(entry.body.clone()))
        .unwrap();

    // Add stale-while-revalidate if configured
    if let Some(stale) = entry.config.stale_while_revalidate {
        let headers = response.headers_mut();
        let cache_control = headers.get("Cache-Control").unwrap().clone();
        let cache_control_str = cache_control.to_str().unwrap().to_string();
        headers.insert(
            "Cache-Control",
            HeaderValue::from_str(&format!(
                "{}, stale-while-revalidate={}",
                cache_control_str, stale
            ))
            .unwrap(),
        );
    }

    // Add cache hit/miss header for debugging
    if is_hit {
        response.headers_mut().insert(
            "X-Cache",
            HeaderValue::from_static("HIT"),
        );
    } else {
        response.headers_mut().insert(
            "X-Cache",
            HeaderValue::from_static("MISS"),
        );
    }

    response
}

/// Cache middleware function
pub async fn cache_layer(
    req: Request,
    next: Next,
) -> Response {
    // For now, just pass through - in a real implementation you'd extract
    // the CacheMiddleware from the request state
    next.run(req).await
}

/// Cache middleware with state
pub async fn cache_layer_with_state(
    axum::extract::State(middleware): axum::extract::State<CacheMiddleware>,
    req: Request,
    next: Next,
) -> Response {
    let key = generate_cache_key(&req);

    // Check cache
    if let Some(entry) = middleware.store.get_fresh(&key).await {
        return build_cached_response(&entry, true);
    }

    // Cache miss - proceed to handler
    let response = next.run(req).await;

    // Extract response body and headers
    let (parts, body) = response.into_parts();

    // Only cache successful responses
    if parts.status.is_success() {
        // Collect body
        let body_bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
            Ok(bytes) => bytes.to_vec(),
            Err(_) => {
                // Failed to read body, rebuild response and return
                return Response::builder()
                    .status(parts.status)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
        };

        // Get content type
        let content_type = parts
            .headers
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        // Get or create cache config from response headers
        let config = extract_cache_config(&parts.headers);

        // Calculate expiration
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = now + config.max_age;

        // Generate ETag
        let etag = parts
            .headers
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(&generate_etag(&body_bytes))
            .to_string();

        let entry = CacheEntry {
            body: body_bytes.clone(),
            content_type,
            expires_at,
            etag,
            config,
        };

        // Store in cache
        middleware.store.insert(key, entry).await;

        // Rebuild response
        let mut response = Response::builder()
            .status(parts.status)
            .body(axum::body::Body::from(body_bytes))
            .unwrap();

        // Copy headers
        *response.headers_mut() = parts.headers;

        // Add cache miss header
        response.headers_mut().insert("X-Cache", HeaderValue::from_static("MISS"));

        response
    } else {
        // Return error response as-is, need to rebuild since we consumed the body
        Response::builder()
            .status(parts.status)
            .body(axum::body::Body::empty())
            .unwrap()
    }
}

/// Extract cache configuration from response headers
fn extract_cache_config(headers: &HeaderMap) -> CacheConfig {
    let mut config = CacheConfig::default();

    if let Some(cache_control) = headers.get("Cache-Control") {
        if let Ok(value) = cache_control.to_str() {
            // Parse max-age
            if let Some(max_age_str) = value
                .split(',')
                .find(|s| s.trim().starts_with("max-age="))
            {
                if let Some(seconds) = max_age_str.split('=').nth(1) {
                    if let Ok(seconds) = seconds.trim().parse::<u64>() {
                        config.max_age = seconds;
                    }
                }
            }

            // Parse stale-while-revalidate
            if let Some(stale_str) = value
                .split(',')
                .find(|s| s.trim().starts_with("stale-while-revalidate="))
            {
                if let Some(seconds) = stale_str.split('=').nth(1) {
                    if let Ok(seconds) = seconds.trim().parse::<u64>() {
                        config.stale_while_revalidate = Some(seconds);
                    }
                }
            }
        }
    }

    // Extract tags from custom header
    if let Some(tags_header) = headers.get("X-Cache-Tags") {
        if let Ok(tags_str) = tags_header.to_str() {
            config.tags = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_cache_store_hit_miss() {
        let store = CacheStore::new();

        // Test cache miss
        assert!(store.get_fresh("test_key").await.is_none());

        // Insert entry
        let entry = CacheEntry {
            body: b"Hello, World!".to_vec(),
            content_type: "text/plain".to_string(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600,
            etag: "test-etag".to_string(),
            config: CacheConfig::default(),
        };

        store.insert("test_key".to_string(), entry).await;

        // Test cache hit
        let retrieved = store.get_fresh("test_key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().body, b"Hello, World!".to_vec());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let store = CacheStore::new();

        // Insert expired entry
        let entry = CacheEntry {
            body: b"Expired".to_vec(),
            content_type: "text/plain".to_string(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - 1, // Expired 1 second ago
            etag: "test-etag".to_string(),
            config: CacheConfig::default(),
        };

        store.insert("expired_key".to_string(), entry).await;

        // Should not return expired entry
        assert!(store.get_fresh("expired_key").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_tag_invalidation() {
        let store = CacheStore::new();

        // Insert entries with tags
        let entry = CacheEntry {
            body: b"Tagged content".to_vec(),
            content_type: "text/plain".to_string(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600,
            etag: "test-etag".to_string(),
            config: CacheConfig {
                tags: vec!["user:123".to_string(), "posts".to_string()],
                ..Default::default()
            },
        };

        store.insert("tagged_key".to_string(), entry).await;

        // Should exist before invalidation
        assert!(store.get_fresh("tagged_key").await.is_some());

        // Invalidate by tag
        store.invalidate_by_tag("user:123").await;

        // Should be removed
        assert!(store.get_fresh("tagged_key").await.is_none());
    }

    #[tokio::test]
    async fn test_generate_cache_key() {
        let request = Request::builder()
            .uri("/test/path")
            .method("GET")
            .body(axum::body::Body::empty())
            .unwrap();

        let key = generate_cache_key(&request);
        assert_eq!(key, "GET:/test/path");
    }

    #[tokio::test]
    async fn test_generate_etag() {
        let content = b"Hello, World!";
        let etag = generate_etag(content);
        assert!(!etag.is_empty());
        assert_eq!(etag.len(), 16); // DefaultHasher produces 16 hex chars
    }

    #[tokio::test]
    async fn test_extract_cache_config() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("max-age=7200, stale-while-revalidate=300"),
        );
        headers.insert(
            "X-Cache-Tags",
            HeaderValue::from_static("user:123,posts"),
        );

        let config = extract_cache_config(&headers);
        assert_eq!(config.max_age, 7200);
        assert_eq!(config.stale_while_revalidate, Some(300));
        assert_eq!(config.tags, vec!["user:123", "posts"]);
    }
}
