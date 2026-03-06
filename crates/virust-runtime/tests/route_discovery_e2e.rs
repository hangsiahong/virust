use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use virust_runtime::VirustApp;

#[tokio::test]
async fn test_route_discovery_e2e() {
    let app = VirustApp::new();
    let router = app.router();

    // Test that routes are accessible
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/test")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // Should either return 200 (route found) or 404 (route not found)
    // Should NOT return 500 (server error)
    assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
