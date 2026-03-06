use anyhow::Result;
use std::fs;
use std::path::Path;
use std::env;

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Get virust workspace path for path dependencies
    let virust_path_absolute = if let Ok(path) = std::env::var("VIRUST_PATH") {
        path
    } else {
        // Default to absolute path of virust workspace (based on binary location)
        // Binary is at target/release/virust, so go up 3 levels to get to repo root
        env::current_exe()
            .ok()
            .and_then(|exe_path| exe_path.canonicalize().ok())
            .and_then(|exe_path| {
                exe_path.parent()
                    .and_then(|parent| parent.parent())
                    .and_then(|parent| parent.parent())
                    .map(|p| p.to_path_buf())
            })
            .and_then(|path| path.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| ".".to_string())
    };

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
lazy_static = "1.4"
chrono = "0.4"
uuid = {{ version = "1.0", features = ["v4"] }}
"#,
        name,
        format!("{}/crates/virust-runtime", virust_path_absolute),
        format!("{}/crates/virust-macros", virust_path_absolute),
        format!("{}/crates/virust-protocol", virust_path_absolute),
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Template-specific setup
    match template {
        "chat" => setup_chat_template(project_dir)?,
        "todo" => setup_todo_template(project_dir)?,
        _ => setup_basic_template(project_dir)?,
    }

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

fn setup_basic_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs that includes api modules
    let lib_rs = r#"pub mod api;

pub use api::chat;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

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

    Ok(())
}

fn setup_chat_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod route;
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with chat implementation
    let route_rs = r#"use virust_macros::ws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ChatEntry {
    pub id: String,
    pub username: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub ok: bool,
    pub message: Option<String>,
}

// Thread-safe message history
lazy_static::lazy_static! {
    static ref MESSAGE_HISTORY: Arc<RwLock<Vec<ChatEntry>>> = Arc::new(RwLock::new(Vec::new()));
}

#[ws]
async fn route(msg: ChatMessage) -> ChatResponse {
    let entry = ChatEntry {
        id: uuid::Uuid::new_v4().to_string(),
        username: msg.username.clone(),
        message: msg.message.clone(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Store message in history
    let mut history = MESSAGE_HISTORY.write().await;
    history.push(entry.clone());

    // Keep only last 100 messages
    if history.len() > 100 {
        history.remove(0);
    }

    println!("[{}] {}: {}", entry.id, msg.username, msg.message);
    ChatResponse {
        ok: true,
        message: Some("Message received".to_string()),
    }
}

/// Get message history
pub async fn history() -> String {
    let history = MESSAGE_HISTORY.read().await;
    serde_json::to_string(&*history).unwrap_or_else(|_| "[]".to_string())
}
"#;
    fs::write(project_dir.join("src/api/route.rs"), route_rs)?;

    // Copy web files from template
    copy_template_files(project_dir, "chat")?;

    Ok(())
}

fn setup_todo_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod todos;
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/todos/route.rs
    let todos_route = r#"use virust_macros::{get, post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use virust_protocol::{Persistence, InMemoryPersistence};

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: i64,
}

// Thread-safe persistence
lazy_static::lazy_static! {
    static ref PERSISTENCE: Arc<InMemoryPersistence> = Arc::new(InMemoryPersistence::new());
}

/// List all todos
#[get]
async fn list_todos() -> String {
    match PERSISTENCE.list("todos").await {
        Ok(todos) => serde_json::to_string(&todos).unwrap_or_else(|_| "[]".to_string()),
        Err(_) => "[]".to_string(),
    }
}

/// Create a new todo
#[post]
async fn create_todo(payload: String) -> String {
    let input: CreateTodoRequest = match serde_json::from_str(&payload) {
        Ok(req) => req,
        Err(_) => return serde_json::json!({"error": "Invalid JSON"}).to_string(),
    };

    let todo = serde_json::json!({
        "title": input.title,
        "description": input.description,
        "completed": false,
        "created_at": chrono::Utc::now().timestamp(),
    });

    match PERSISTENCE.create("todos", todo).await {
        Ok(id) => {
            let response = serde_json::json!({
                "id": id,
                "title": input.title,
                "description": input.description,
                "completed": false,
                "created_at": chrono::Utc::now().timestamp(),
            });
            serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
        }
        Err(e) => {
            serde_json::json!({"error": e.to_string()}).to_string()
        }
    }
}
"#;
    fs::write(project_dir.join("src/api/todos.rs"), todos_route)?;

    // Create api/todos_id/route.rs for individual todo operations
    fs::create_dir_all(project_dir.join("src/api/todos_id"))?;
    let todos_id_route = r#"use virust_macros::{get, put, delete};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use virust_protocol::{Persistence, InMemoryPersistence};

#[derive(Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: i64,
}

// Thread-safe persistence (shared with todos.rs)
lazy_static::lazy_static! {
    static ref PERSISTENCE: Arc<InMemoryPersistence> = Arc::new(InMemoryPersistence::new());
}

/// Get a specific todo by ID
#[get]
async fn get_todo(id: String) -> String {
    match PERSISTENCE.get::<Value>("todos", &id).await {
        Ok(Some(todo)) => serde_json::to_string(&todo).unwrap_or_else(|_| "{}".to_string()),
        Ok(None) => serde_json::json!({"error": "Todo not found"}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Update a todo by ID
#[put]
async fn update_todo(id: String, payload: String) -> String {
    let input: UpdateTodoRequest = match serde_json::from_str(&payload) {
        Ok(req) => req,
        Err(_) => return serde_json::json!({"error": "Invalid JSON"}).to_string(),
    };

    // First get the existing todo
    let existing = match PERSISTENCE.get::<Value>("todos", &id).await {
        Ok(Some(todo)) => todo,
        Ok(None) => return serde_json::json!({"error": "Todo not found"}).to_string(),
        Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
    };

    // Update fields if provided
    let mut updated = existing.clone();
    if let Some(title) = &input.title {
        updated["title"] = serde_json::Value::String(title.clone());
    }
    if let Some(description) = &input.description {
        updated["description"] = if let Some(desc) = description {
            serde_json::Value::String(desc.clone())
        } else {
            serde_json::Value::Null
        };
    }
    if let Some(completed) = input.completed {
        updated["completed"] = serde_json::Value::Bool(completed);
    }

    match PERSISTENCE.update("todos", &id, updated.clone()).await {
        Ok(_) => serde_json::to_string(&updated).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Delete a todo by ID
#[delete]
async fn delete_todo(id: String) -> String {
    match PERSISTENCE.delete("todos", &id).await {
        Ok(_) => serde_json::json!({"success": true}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
"#;
    fs::write(project_dir.join("src/api/todos_id/route.rs"), todos_id_route)?;

    // Copy web files from template
    copy_template_files(project_dir, "todo")?;

    Ok(())
}

fn copy_template_files(project_dir: &Path, template_name: &str) -> Result<()> {
    match template_name {
        "chat" => {
            fs::write(project_dir.join("web/index.html"), include_str!("../templates/chat/web/index.html"))?;
            fs::write(project_dir.join("web/main.js"), include_str!("../templates/chat/web/main.js"))?;
            fs::write(project_dir.join("web/styles.css"), include_str!("../templates/chat/web/styles.css"))?;
        }
        "todo" => {
            fs::write(project_dir.join("web/index.html"), include_str!("../templates/todo/web/index.html"))?;
            fs::write(project_dir.join("web/main.js"), include_str!("../templates/todo/web/main.js"))?;
            fs::write(project_dir.join("web/styles.css"), include_str!("../templates/todo/web/styles.css"))?;
        }
        _ => {}
    }
    Ok(())
}
