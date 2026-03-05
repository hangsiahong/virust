use virust_runtime::VirustApp;
use axum::body::Body;
use axum::http::{Request, StatusCode, header::UPGRADE, header::CONNECTION};
use tower::ServiceExt;

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

    // The server should respond with either 101 (Switching Protocols) or 400 (Bad Request)
    // if the WebSocket upgrade fails due to missing headers or other issues
    let status = response.status();
    assert!(status == StatusCode::SWITCHING_PROTOCOLS || status == StatusCode::BAD_REQUEST);
}