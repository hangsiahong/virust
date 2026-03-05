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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = VirustApp::new();
    let router = app.router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🚀 Server running on http://127.0.0.1:3000");

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
</body>
</html>
"#;
    fs::write(project_dir.join("web/index.html"), index_html)?;

    println!("✓ Created project '{}'", name);
    println!("✓ Template: {}", template);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  cargo run");
    println!();
    println!("Note: This project uses path dependencies to the local virust workspace.");
    println!("      Set VIRUST_PATH environment variable if the virust crates are in a custom location.");

    Ok(())
}