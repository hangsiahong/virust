use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Get virust workspace path for path dependencies
    let virust_path = std::env::var("VIRUST_PATH").unwrap_or_else(|_| {
        // If VIRUST_PATH not set, try to use relative path from current dir
        "../..".to_string()
    });

    // Create project structure
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("src/api"))?;
    fs::create_dir_all(project_dir.join("web"))?;

    // Create Cargo.toml with path dependencies for local development
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
virust-runtime = {{ path = "{}" }}
virust-macros = {{ path = "{}" }}
virust-protocol = {{ path = "{}" }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tokio = {{ version = "1", features = ["full"] }}
inventory = "0.3"
axum = "0.7"
anyhow = "1"
"#,
        name,
        format!("{}/crates/virust-runtime", virust_path),
        format!("{}/crates/virust-macros", virust_path),
        format!("{}/crates/virust-protocol", virust_path),
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create lib.rs that includes api modules
    let lib_rs = r#"pub mod api;

pub use api::chat;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create main.rs as entry point
    let main_rs = r#"use virust_runtime::VirustApp;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = args.iter()
        .position(|x| x == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let app = VirustApp::new();
    let router = app.router();

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("🚀 Server running on http://127.0.0.1:{}", port);

    axum::serve(listener, router).await?;

    Ok(())
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // Create api/mod.rs to export route modules
    let api_mod = r#"pub mod chat;
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create example route
    let route_rs = r#"use virust_macros::ws;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub ok: bool,
}

#[ws]
async fn chat(msg: ChatMessage) -> ChatResponse {
    println!("Received: {}", msg.message);
    ChatResponse { ok: true }
}
"#;
    fs::write(project_dir.join("src/api/chat.rs"), route_rs)?;

    // Create a simple web/index.html
    let index_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Virust App</title>
</head>
<body>
    <h1>Welcome to Virust!</h1>
    <p>Edit web/index.html to change this page.</p>
    <script type="module" src="/main.js"></script>
</body>
</html>
"#;
    fs::write(project_dir.join("web/index.html"), index_html)?;

    // Create web/main.js
    let main_js = r#"console.log('Virust app initialized');

// You can now fetch types from the backend:
// fetch('/api/__types')
//   .then(r => r.json())
//   .then(types => console.log(types));
"#;
    fs::write(project_dir.join("web/main.js"), main_js)?;

    // Create vite.config.ts
    let vite_config = r#"import { defineConfig } from 'vite'

export default defineConfig({
  server: {
    port: 5173,
    proxy: {
      // Proxy API requests to backend
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
      // Proxy WebSocket connections to backend
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true,
      },
    },
  },
})
"#;
    fs::write(project_dir.join("web/vite.config.ts"), vite_config)?;

    // Create web/package.json
    let package_json = r#"{
  "name": "virust-app",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}"#;
    fs::write(project_dir.join("web/package.json"), package_json)?;

    println!("✓ Created project '{}'", name);
    println!("✓ Template: {}", template);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  npm install  # Install frontend dependencies");
    println!("  virust dev  # Start both frontend and backend");
    println!();
    println!("Note: This project uses path dependencies to the local virust workspace.");
    println!("      Set VIRUST_PATH environment variable if the virust crates are in a custom location.");

    Ok(())
}