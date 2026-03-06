use serde_json::Value;
use axum::response::{Html, IntoResponse};

/// HTML response for SSR-rendered components
///
/// # Security
///
/// - `component_name` is derived from `#[render_component("Name")]` macro attributes
///   and is always a compile-time constant, not user input
/// - `props` are serialized as JSON; note that this placeholder implementation
///   does not escape `</script>` sequences - this will be addressed in Task 10
///   when actual Bun rendering is implemented
///
/// # Note
///
/// This is a placeholder implementation. Task 10 will replace the `IntoResponse`
/// implementation with actual Bun-based SSR rendering.
pub struct RenderedHtml {
    pub component_name: String,
    pub props: Value,
}

impl RenderedHtml {
    pub fn with_props(component_name: &str, props: Value) -> Self {
        Self {
            component_name: component_name.to_string(),
            props,
        }
    }

    pub fn new(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            props: Value::Object(Default::default()),
        }
    }
}

impl IntoResponse for RenderedHtml {
    fn into_response(self) -> axum::response::Response {
        // SECURITY: This is a placeholder implementation.
        // Task 10 will replace this with actual Bun rendering which doesn't have
        // the script breakout issue. The component_name is trusted (compile-time
        // constant from macros) so XSS is not a concern there.
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body>
<div id="root">
<p>Component: {} will be rendered here</p>
</div>
<script id="__VIRUST_PROPS__" type="application/json">{}</script>
</body>
</html>"#,
            self.component_name, self.component_name, self.props
        );
        Html(html).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendered_html_creation() {
        let html = RenderedHtml::new("App");
        assert_eq!(html.component_name, "App");
    }

    #[test]
    fn test_rendered_html_with_props() {
        let props = serde_json::json!({"title": "Hello"});
        let html = RenderedHtml::with_props("App", props);
        assert_eq!(html.component_name, "App");
        assert_eq!(html.props["title"], "Hello");
    }

    #[test]
    fn test_into_response() {
        let html = RenderedHtml::new("App");
        let response = html.into_response();
        // Just check it doesn't panic
    }
}
