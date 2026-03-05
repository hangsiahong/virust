use virust_runtime::registry::RouteRegistry;
use virust_protocol::{HttpRequest, HttpResponse};
use std::sync::Arc;

#[tokio::test]
async fn test_registry_empty() {
    let registry = RouteRegistry::new();
    assert!(registry.get_http("/api/test").is_none());
}

#[tokio::test]
async fn test_register_http_route() {
    let mut registry = RouteRegistry::new();

    registry.register_http(
        "/api/test".to_string(),
        Arc::new(|_req: HttpRequest| {
            HttpResponse::ok()
        })
    );

    assert!(registry.get_http("/api/test").is_some());
}
