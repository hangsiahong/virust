use virust_runtime::VirustApp;
use axum::body::Body;
use axum::http::{Request, StatusCode, header::UPGRADE, header::CONNECTION};
use tower_http::ServiceExt;

#[tokio::test]
async fn test_websocket_upgrade() {
    let app = VirustApp::new();
    let router = app.router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/ws")
                .header(UPGRADE, "websocket")
                .header(CONNECTION, "upgrade")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
}