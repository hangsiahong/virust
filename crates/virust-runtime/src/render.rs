use serde_json::Value;
use axum::response::{Html, IntoResponse};
use tokio::sync::mpsc;

pub struct RenderedHtml {
    pub component_name: String,
    pub props: Value,
}

// Channel for sending render requests
lazy_static::lazy_static! {
    static ref RENDER_TX: mpsc::Sender<RenderRequest> = {
        let (tx, mut rx) = mpsc::channel::<RenderRequest>(100);

        tokio::spawn(async move {
            let mut renderer_opt: Option<::virust_bun::BunRenderer> = None;

            while let Some(req) = rx.recv().await {
                // Ensure renderer is running
                let needs_restart = if renderer_opt.is_none() {
                    true
                } else {
                    !renderer_opt.as_mut().unwrap().is_alive()
                };

                if needs_restart {
                    if let Ok(r) = ::virust_bun::BunRenderer::new() {
                        renderer_opt = Some(r);
                    }
                }

                if let Some(ref mut renderer) = renderer_opt {
                    match renderer.render_component(&req.component_name, req.props).await {
                        Ok(output) => {
                            let _ = req.tx.send(Ok(output));
                        }
                        Err(e) => {
                            let _ = req.tx.send(Err(anyhow::anyhow!("Render failed: {}", e)));
                        }
                    }
                } else {
                    let _ = req.tx.send(Err(anyhow::anyhow!("Failed to initialize Bun")));
                }
            }
        });

        tx
    };
}

struct RenderRequest {
    component_name: String,
    props: Value,
    tx: mpsc::Sender<Result<::virust_bun::RenderedOutput, anyhow::Error>>,
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
        let (tx, mut rx) = mpsc::channel(1);

        RENDER_TX.send(RenderRequest {
            component_name: self.component_name.clone(),
            props: self.props.clone(),
            tx,
        }).await?;

        let output = rx.recv().await.ok_or_else(|| anyhow::anyhow!("No response"))??;

        Ok(self.wrap_html(output.html, output.hydration_data))
    }

    fn wrap_html(&self, body: String, hydration_data: String) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <script crossorigin src="https://unpkg.com/react@18/umd/react.production.min.js"></script>
    <script crossorigin src="https://unpkg.com/react-dom@18/umd/react-dom.production.min.js"></script>
</head>
<body>
    <div id="root">{}</div>
    <script type="application/json" id="__VIRUST_PROPS__">{}</script>
    <script type="module" src="/bun/client.js"></script>
</body>
</html>"#,
            self.component_name,
            body,
            hydration_data
        )
    }
}

impl IntoResponse for RenderedHtml {
    fn into_response(self) -> axum::response::Response {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(self.render()) {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                eprintln!("SSR Error: {}", e);
                Html("<html><body><h1>SSR Error</h1></body></html>").into_response()
            }
        }
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
    #[ignore] // Requires Bun
    async fn test_render_with_bun() {
        let html = RenderedHtml::new("Test");
        let _result = html.render().await;
        // Will test once we have actual components
    }
}
