use virust_bun::BunRenderer;
use serde_json::json;

#[tokio::test]
#[ignore] // Requires Bun to be installed
async fn test_render_simple_component() {
    let renderer_path = "bundled/renderer.js";
    let mut renderer = BunRenderer::with_path(renderer_path).unwrap();

    // Set up test component
    let web_dir = std::path::PathBuf::from("tests/fixtures/web");
    renderer.set_web_dir(&web_dir).unwrap();

    let props = json!({"name": "World"});
    let output = renderer.render_component("HelloWorld", props).await.unwrap();

    assert!(output.html.contains("Hello World"));
}

#[tokio::test]
#[ignore]
async fn test_server_component_rendering() {
    let renderer_path = "bundled/renderer.js";
    let mut renderer = BunRenderer::with_path(renderer_path).unwrap();
    let web_dir = std::path::PathBuf::from("tests/fixtures/web");
    renderer.set_web_dir(&web_dir).unwrap();

    let output = renderer.render_component("ServerComponent", json!({"message": "Hello"})).await.unwrap();
    assert!(output.html.contains("server"));
    assert!(output.html.contains("Hello"));
}

#[tokio::test]
#[ignore]
async fn test_client_component_placeholder() {
    let renderer_path = "bundled/renderer.js";
    let mut renderer = BunRenderer::with_path(renderer_path).unwrap();
    let web_dir = std::path::PathBuf::from("tests/fixtures/web");
    renderer.set_web_dir(&web_dir).unwrap();

    let output = renderer.render_component("ClientComponent", json!({"initialCount": 0})).await.unwrap();
    assert!(output.html.contains("data-client"));
    assert!(output.html.contains("data-component"));
}
