use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create project structure
    fs::create_dir_all(project_dir.join("api"))?;
    fs::create_dir_all(project_dir.join("web"))?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
virust-runtime = "0.1.0"
virust-macros = "0.1.0"
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        name
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create example route
    let route_rs = format!(
        r#"use virust_macros::ws;
use serde::{{Deserialize, Serialize}};

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {{
    pub message: String,
}}

#[derive(Serialize)]
pub struct ChatResponse {{
    pub ok: bool,
}}

#[ws]
async fn chat(msg: ChatMessage) -> ChatResponse {{
    println!("Received: {{}}", msg.message);
    ChatResponse {{ ok: true }}
}}
"#
    );
    fs::write(project_dir.join("api/chat.rs"), route_rs)?;

    println!("✓ Created project '{}'", name);
    println!("✓ Template: {}", template);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  virust dev");

    Ok(())
}