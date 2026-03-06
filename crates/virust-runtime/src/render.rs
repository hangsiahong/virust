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
    let mut renderer = ::virust_bun::BunRenderer::new()?;
    renderer.set_web_dir(web_dir)?;
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

    pub async fn render(self) -> Result<String, anyhow::Error> {
        let guard = BUN_RENDERER.read().await;

        if let Some(_renderer) = guard.as_ref() {
            // We need mutable access, so we'll use a different approach
            // For now, return placeholder
            Ok(format!(
                "<div>Component: {} (SSR not yet implemented)</div>",
                self.component_name
            ))
        } else {
            Ok("<div>Bun renderer not initialized</div>".to_string())
        }
    }
}

// Async renderer helper
pub struct BunRendererExtension(pub Arc<RwLock<Option<::virust_bun::BunRenderer>>>);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for BunRendererExtension
where
    S: Send + Sync,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(BunRendererExtension(Arc::clone(&BUN_RENDERER)))
    }
}

impl IntoResponse for RenderedHtml {
    fn into_response(self) -> axum::response::Response {
        // This now needs to be async, so we'll use a different approach
        // Return a response that will be rendered by middleware
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Virust SSR</title></head>
<body>
<div id="root">
<p>Component: {}</p>
<p>Props: {}</p>
</div>
<script id="__VIRUST_PROPS__" type="application/json">{}</script>
</body>
</html>"#,
            self.component_name,
            self.props,
            self.props
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
        let _response = html.into_response();
        // Just check it doesn't panic
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[tokio::test]
    async fn test_render_without_bun() {
        let html = RenderedHtml::new("Test");
        let result = html.render().await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Bun renderer not initialized"));
    }
}
