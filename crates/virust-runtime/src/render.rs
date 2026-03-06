use serde_json::Value;
use axum::response::{Html, IntoResponse};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RenderedHtml {
    pub component_name: String,
    pub props: Value,
}

// Global Bun renderer instance
lazy_static::lazy_static! {
    static ref BUN_RENDERER: Arc<RwLock<Option<::virust_bun::BunRenderer>>> =
        Arc::new(RwLock::new(None));
}

pub async fn init_bun_renderer(web_dir: &std::path::Path) -> Result<(), anyhow::Error> {
    let renderer = ::virust_bun::BunRenderer::new()?;
    let mut guard = BUN_RENDERER.write().await;
    *guard = Some(renderer);
    Ok(())
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

    pub async fn render_to_response(self) -> axum::response::Response {
        let guard = BUN_RENDERER.read().await;

        if let Some(_renderer) = guard.as_ref() {
            // Clone renderer since we need mutable access
            // This will be fixed with proper locking in Task 11
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body>
<div id="root">
<p>Component: {} with props: {}</p>
</div>
<script id="__VIRUST_PROPS__" type="application/json">{}</script>
</body>
</html>"#,
                self.component_name,
                self.component_name,
                self.props,
                self.props
            );
            Html(html).into_response()
        } else {
            // Fallback if Bun not initialized
            Html("<html><body><p>Bun renderer not initialized</p></body></html>").into_response()
        }
    }
}

impl IntoResponse for RenderedHtml {
    fn into_response(self) -> axum::response::Response {
        // Convert to async response
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.render_to_response())
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
