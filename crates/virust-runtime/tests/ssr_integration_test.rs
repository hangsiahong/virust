use virust_runtime::RenderedHtml;
use axum::response::IntoResponse;

#[test]
fn test_rendered_html() {
    let html = RenderedHtml::new("App");
    let _response = html.into_response();
    // Verify response is created
    assert!(true); // Placeholder test
}
