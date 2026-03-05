use virust_runtime::VirustApp;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower_http::ServiceExt;

#[tokio::test]
async fn test_basic_http_server() {
    let app = VirustApp::new();
    let router = app.router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}