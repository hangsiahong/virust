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
            eprintln!("SSR: Render task started");
            let mut renderer_opt: Option<::virust_bun::BunRenderer> = None;

            while let Some(req) = rx.recv().await {
                eprintln!("SSR: Render task received request for component: {}", req.component_name);

                // Ensure renderer is running
                let needs_restart = if renderer_opt.is_none() {
                    eprintln!("SSR: No renderer, creating new one");
                    true
                } else {
                    !renderer_opt.as_mut().unwrap().is_alive()
                };

                if needs_restart {
                    eprintln!("SSR: Initializing Bun renderer");
                    match ::virust_bun::BunRenderer::new() {
                        Ok(r) => {
                            renderer_opt = Some(r);
                            eprintln!("SSR: Bun renderer initialized successfully");
                        }
                        Err(e) => {
                            eprintln!("SSR: Failed to initialize Bun: {}", e);
                            let _ = req.tx.send(Err(anyhow::anyhow!("Failed to initialize Bun: {}", e)));
                            continue;
                        }
                    }
                }

                if let Some(ref mut renderer) = renderer_opt {
                    eprintln!("SSR: Calling render_component for '{}'", req.component_name);
                    match renderer.render_component(&req.component_name, req.props).await {
                        Ok(output) => {
                            eprintln!("SSR: Render successful, HTML length: {}", output.html.len());
                            let _ = req.tx.send(Ok(output));
                        }
                        Err(e) => {
                            eprintln!("SSR: Render failed: {}", e);
                            let _ = req.tx.send(Err(anyhow::anyhow!("Render failed: {}", e)));
                        }
                    }
                } else {
                    eprintln!("SSR: No renderer available");
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

    fn error_page(&self, error: &anyhow::Error) -> String {
        let error_message = error.to_string();
        let error_chain: String = error.chain().map(|e| e.to_string()).collect::<Vec<_>>().join("\nCaused by: ");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SSR Error - {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }}
        .error-container {{
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            max-width: 800px;
            width: 100%;
            overflow: hidden;
        }}
        .error-header {{
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .error-header h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
            font-weight: 700;
        }}
        .error-header p {{
            font-size: 1.1em;
            opacity: 0.9;
        }}
        .error-body {{
            padding: 30px;
        }}
        .error-section {{
            margin-bottom: 25px;
        }}
        .error-section h3 {{
            color: #333;
            font-size: 1.2em;
            margin-bottom: 10px;
            font-weight: 600;
        }}
        .component-name {{
            background: #f7f7f7;
            padding: 15px;
            border-radius: 8px;
            font-family: 'Courier New', monospace;
            font-size: 1.1em;
            color: #667eea;
            font-weight: 600;
        }}
        .error-message {{
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 15px;
            border-radius: 4px;
            color: #856404;
            line-height: 1.6;
        }}
        details {{
            background: #f8f9fa;
            border-radius: 8px;
            overflow: hidden;
        }}
        summary {{
            padding: 15px;
            cursor: pointer;
            font-weight: 600;
            color: #495057;
            user-select: none;
            transition: background 0.2s;
        }}
        summary:hover {{
            background: #e9ecef;
        }}
        .stack-trace {{
            padding: 15px;
            border-top: 1px solid #dee2e6;
        }}
        .stack-trace pre {{
            margin: 0;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
            color: #c7254e;
            white-space: pre-wrap;
            word-wrap: break-word;
            line-height: 1.5;
        }}
        .help-section {{
            background: #d1ecf1;
            border-left: 4px solid #17a2b8;
            padding: 15px;
            border-radius: 4px;
            color: #0c5460;
        }}
        .help-section ul {{
            margin-left: 20px;
            margin-top: 10px;
        }}
        .help-section li {{
            margin-bottom: 5px;
        }}
        .help-section a {{
            color: #0c5460;
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <div class="error-header">
            <h1>⚠️ Render Error</h1>
            <p>Server-Side Rendering Failed</p>
        </div>
        <div class="error-body">
            <div class="error-section">
                <h3>Component:</h3>
                <div class="component-name">{}</div>
            </div>
            <div class="error-section">
                <h3>Error Message:</h3>
                <div class="error-message">{}</div>
            </div>
            <div class="error-section">
                <h3>Stack Trace:</h3>
                <details>
                    <summary>Click to view full stack trace</summary>
                    <div class="stack-trace">
                        <pre>{}</pre>
                    </div>
                </details>
            </div>
            <div class="error-section">
                <h3>What to do:</h3>
                <div class="help-section">
                    <p>This error occurred during server-side rendering. To fix this issue:</p>
                    <ul>
                        <li>Check the server logs for more detailed error information</li>
                        <li>Verify that the component exists and is properly exported</li>
                        <li>Ensure all component dependencies are available</li>
                        <li>Check for JavaScript syntax errors in your component</li>
                    </ul>
                    <p style="margin-top: 10px;"><strong>Note:</strong> This detailed error page is intended for development environments to help with debugging.</p>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
            self.component_name,
            self.component_name,
            html_escape::encode_text(&error_message),
            html_escape::encode_text(&error_chain)
        )
    }

    pub async fn render(self) -> Result<String, anyhow::Error> {
        eprintln!("SSR: Rendering component '{}'", self.component_name);

        let (tx, mut rx) = mpsc::channel(1);

        RENDER_TX.send(RenderRequest {
            component_name: self.component_name.clone(),
            props: self.props.clone(),
            tx,
        }).await?;
        eprintln!("SSR: Request sent to render task");

        let output = rx.recv().await.ok_or_else(|| anyhow::anyhow!("No response from render task"))??;
        eprintln!("SSR: Got response, HTML length: {}", output.html.len());

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
        let component_name = self.component_name.clone();
        let props = self.props.clone();
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(self.render()) {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                // Log error to stderr for server-side visibility
                eprintln!("SSR Error for component '{}': {}", component_name, e);
                for cause in e.chain().skip(1) {
                    eprintln!("  Caused by: {}", cause);
                }

                // Create a new RenderedHtml for error page generation
                let error_html = RenderedHtml {
                    component_name,
                    props,
                };
                // Return development-friendly error page
                Html(error_html.error_page(&e)).into_response()
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

    #[test]
    fn test_error_page_contains_component_name() {
        let html = RenderedHtml::new("NonExistentComponent");
        let error = anyhow::anyhow!("Test error message");
        let error_html = html.error_page(&error);

        // Verify the error page contains the component name
        assert!(error_html.contains("NonExistentComponent"));
        // Verify it contains the error message
        assert!(error_html.contains("Test error message"));
        // Verify it has the error page structure
        assert!(error_html.contains("error-container"));
        assert!(error_html.contains("error-header"));
        assert!(error_html.contains("error-body"));
    }

    #[test]
    fn test_error_page_with_error_chain() {
        let html = RenderedHtml::new("TestComponent");
        let error = anyhow::anyhow!("Outer error")
            .context(anyhow::anyhow!("Inner error 1"));
        let error_html = html.error_page(&error);

        // Verify the error page contains the full error chain
        assert!(error_html.contains("Outer error"));
        assert!(error_html.contains("Inner error 1"));
        // Verify stack trace section exists
        assert!(error_html.contains("Stack Trace"));
        assert!(error_html.contains("<details>"));
    }

    #[test]
    fn test_into_response_with_invalid_component() {
        // This test verifies that even with a non-existent component,
        // into_response doesn't panic and returns valid HTML
        let html = RenderedHtml::new("DefinitelyNotARealComponent");
        let response = html.into_response();

        // The response should be valid (not panic)
        // In development mode with Bun not running, this will return an error page
        // The important thing is it doesn't crash
        assert!(response.status().is_success() || response.status().is_server_error());
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
