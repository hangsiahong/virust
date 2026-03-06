use virust_bun::BunRenderer;
use serde_json::json;

#[tokio::test]
#[ignore] // Requires Bun to be installed
async fn test_render_simple_component() {
    let mut renderer = BunRenderer::new().unwrap();

    // Set up test component
    let web_dir = std::path::PathBuf::from("tests/fixtures/web");
    renderer.set_web_dir(&web_dir).unwrap();

    let props = json!({"name": "World"});
    let output = renderer.render_component("HelloWorld", props).await.unwrap();

    assert!(output.html.contains("Hello World"));
}
