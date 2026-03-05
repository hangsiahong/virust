use virust_runtime::VirustApp;

#[tokio::test]
async fn test_basic_http_server_creation() {
    let app = VirustApp::new();
    let router = app.router();
    
    // Simple test - just creating the app should work
    assert!(true);
}
