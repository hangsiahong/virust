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
        "ssr-blog" => setup_ssr_blog_template(project_dir)?,
        _ => setup_basic_template(project_dir)?,
    }

    // Create main.rs as entry point
    let main_rs = format!(
        r#"use virust_runtime::VirustApp;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let args: Vec<String> = env::args().collect();
    let port = args.iter()
        .position(|x| x == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let app = VirustApp::new();
    let router = app.router();

    // Register user routes from the api module
    let router = {project_name}::api::register_routes(router);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{{}}", port)).await?;
    println!("🚀 Server running on http://127.0.0.1:{{}}", port);

    axum::serve(listener, router).await?;

    Ok(())
}}
"#,
        project_name = name.replace("-", "_")
    );
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // No longer need vite.config.ts - using pure static file serving

    // Create minimal web/package.json (no vite dependency)
    let package_json = r#"{
  "name": "virust-app",
  "version": "0.1.0"
}"#;
    fs::write(project_dir.join("web/package.json"), package_json)?;

    println!("✓ Created project '{}'", name);
    println!("✓ Template: {}", template);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  virust dev  # Start server on http://127.0.0.1:3000");
    println!();
    println!("Note: This project uses path dependencies to the local virust workspace.");
    println!("      Set VIRUST_PATH environment variable if the virust crates are in a custom location.");

    Ok(())
}

fn setup_basic_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs that includes api modules
    let lib_rs = r#"pub mod api;

pub use api::route;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs to export route modules
    let api_mod = r#"pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::post;

    // Register echo endpoint
    router.route("/api/echo", post(route::route_wrapper))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create example route
    let route_rs = r#"use virust_macros::post;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct EchoMessage {
    pub message: String,
}

#[derive(Serialize)]
pub struct EchoResponse {
    pub ok: bool,
    pub echoed: String,
}

#[post]
pub async fn route(#[body] msg: EchoMessage) -> axum::Json<EchoResponse> {
    println!("Received: {}", msg.message);
    axum::Json(EchoResponse {
        ok: true,
        echoed: msg.message,
    })
}
"#;
    fs::write(project_dir.join("src/api/route.rs"), route_rs)?;

    // Create a simple web/index.html with example usage
    let index_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Virust App - Basic Template</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        .container {
            display: flex;
            flex-direction: column;
            gap: 15px;
        }
        input, button {
            padding: 10px;
            font-size: 16px;
        }
        button {
            background: #007bff;
            color: white;
            border: none;
            cursor: pointer;
        }
        button:hover {
            background: #0056b3;
        }
        .response {
            margin-top: 20px;
            padding: 15px;
            background: #f5f5f5;
            border-radius: 5px;
        }
    </style>
</head>
<body>
    <h1>Welcome to Virust!</h1>
    <p>This is a basic template with a simple echo endpoint.</p>

    <div class="container">
        <input type="text" id="messageInput" placeholder="Type a message..." />
        <button onclick="sendMessage()">Send Message</button>
        <div class="response">
            <strong>Response:</strong>
            <pre id="response">Waiting for input...</pre>
        </div>
    </div>

    <script type="module" src="/main.js"></script>
    <script>
        async function sendMessage() {
            const input = document.getElementById('messageInput');
            const response = document.getElementById('response');
            const message = input.value.trim();

            if (!message) {
                response.textContent = 'Please enter a message';
                return;
            }

            try {
                const res = await fetch('/api/echo', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ message })
                });
                const data = await res.json();
                response.textContent = JSON.stringify(data, null, 2);
                input.value = '';
            } catch (err) {
                response.textContent = 'Error: ' + err.message;
            }
        }

        // Allow Enter key to send message
        document.getElementById('messageInput').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendMessage();
        });
    </script>
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

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::{get, post};

    // Register chat message endpoint and history
    router
        .route("/api/chat", post(route::route_wrapper))
        .route("/api/chat/history", get(route::history))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with chat implementation
    let route_rs = r#"use virust_macros::post;
use virust_macros::body;
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

#[post]
pub async fn route(#[body] msg: ChatMessage) -> axum::Json<ChatResponse> {
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
    axum::Json(ChatResponse {
        ok: true,
        message: Some("Message received".to_string()),
    })
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
pub mod persistence;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create persistence.rs with shared PERSISTENCE
    let persistence_rs = r#"use std::sync::Arc;
use virust_protocol::InMemoryPersistence;

// Thread-safe persistence shared across all API modules
lazy_static::lazy_static! {
    pub static ref PERSISTENCE: Arc<InMemoryPersistence> = Arc::new(InMemoryPersistence::new());
}
"#;
    fs::write(project_dir.join("src/persistence.rs"), persistence_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod todos;
pub mod todos_id;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::{get, post, put, delete};

    // Register todos routes
    // Note: list_todos has no parameters, so it's called directly
    // Other handlers have path/body parameters, so they use _wrapper suffix
    router
        .route("/api/todos", get(todos::list_todos))
        .route("/api/todos", post(todos::create_todo_wrapper))
        // Register todos_id routes (all have path parameters, so use _wrapper)
        .route("/api/todos/:id", get(todos_id::get_todo_wrapper))
        .route("/api/todos/:id", put(todos_id::update_todo_wrapper))
        .route("/api/todos/:id", delete(todos_id::delete_todo_wrapper))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/todos.rs
    let todos_route = r#"use virust_macros::{get, post};
use serde::{Deserialize, Serialize};
use virust_protocol::Persistence;
use crate::persistence::PERSISTENCE;

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

/// List all todos
#[get]
pub async fn list_todos() -> String {
    match PERSISTENCE.list("todos").await {
        Ok(todos) => serde_json::to_string(&todos).unwrap_or_else(|_| "[]".to_string()),
        Err(_) => "[]".to_string(),
    }
}

/// Create a new todo
#[post]
pub async fn create_todo(#[body] input: CreateTodoRequest) -> String {
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

    // Create api/todos_id.rs for individual todo operations
    let todos_id_route = r#"use virust_macros::{get, put, delete};
use serde::{Deserialize, Serialize};
use virust_protocol::Persistence;
use crate::persistence::PERSISTENCE;

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

/// Get a specific todo by ID
#[get]
pub async fn get_todo(#[path] id: String) -> String {
    match PERSISTENCE.get("todos", &id).await {
        Ok(Some(todo)) => serde_json::to_string(&todo).unwrap_or_else(|_| "{}".to_string()),
        Ok(None) => serde_json::json!({"error": "Todo not found"}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Update a todo by ID
#[put]
pub async fn update_todo(#[path] id: String, #[body] input: UpdateTodoRequest) -> String {
    // First get the existing todo
    let existing = match PERSISTENCE.get("todos", &id).await {
        Ok(Some(todo)) => todo,
        Ok(None) => return serde_json::json!({"error": "Todo not found"}).to_string(),
        Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
    };

    // Update fields if provided
    let mut updated = existing.clone();
    if let Some(title) = &input.title {
        updated["title"] = serde_json::Value::String(title.clone());
    }
    // Handle description field - can be set to a value or explicitly to null
    if input.description.is_some() {
        updated["description"] = match &input.description {
            Some(desc) => serde_json::Value::String(desc.clone()),
            None => serde_json::Value::Null,
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
pub async fn delete_todo(#[path] id: String) -> String {
    match PERSISTENCE.delete("todos", &id).await {
        Ok(_) => serde_json::json!({"success": true}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
"#;
    fs::write(project_dir.join("src/api/todos_id.rs"), todos_id_route)?;

    // Copy web files from template
    copy_template_files(project_dir, "todo")?;

    Ok(())
}

fn setup_ssr_blog_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::get;

    // Register home page route with SSR
    router.route("/", get(route::home_wrapper))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with SSR implementation
    let route_rs = r#"use virust_macros::{get, render_component};
use virust_runtime::RenderedHtml;

/// Home page with server-side rendering
#[get]
#[render_component("HomePage")]
pub async fn home() -> RenderedHtml {
    RenderedHtml::new("HomePage")
}
"#;
    fs::write(project_dir.join("src/api/route.rs"), route_rs)?;

    // Create web/components directory
    fs::create_dir_all(project_dir.join("web/components"))?;

    // Create HomePage.jsx component
    let home_page_jsx = r#"// HomePage.jsx - Server-side rendered blog home page
export default async function HomePage() {
  // This component is rendered on the server
  // In a real blog, you might fetch posts here

  return (
    <div style={{
      maxWidth: '800px',
      margin: '50px auto',
      padding: '20px',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <header style={{
        marginBottom: '40px',
        paddingBottom: '20px',
        borderBottom: '1px solid #eaeaea'
      }}>
        <h1 style={{ fontSize: '2.5rem', marginBottom: '10px' }}>
          Welcome to My Blog
        </h1>
        <p style={{ color: '#666', fontSize: '1.1rem' }}>
          Built with Virust SSR
        </p>
      </header>

      <main>
        <section style={{ marginBottom: '40px' }}>
          <h2 style={{ fontSize: '1.8rem', marginBottom: '20px' }}>
            Latest Posts
          </h2>

          <article style={{
            padding: '20px',
            background: '#f9f9f9',
            borderRadius: '8px',
            marginBottom: '20px'
          }}>
            <h3 style={{ fontSize: '1.3rem', marginBottom: '10px' }}>
              Getting Started with Virust
            </h3>
            <p style={{ color: '#666', lineHeight: '1.6' }}>
              This is a server-side rendered blog post. The HTML is generated on the server
              and sent to the browser, providing fast initial page loads and excellent SEO.
            </p>
            <div style={{ marginTop: '15px', fontSize: '0.9rem', color: '#888' }}>
              Posted on March 6, 2026
            </div>
          </article>

          <article style={{
            padding: '20px',
            background: '#f9f9f9',
            borderRadius: '8px',
            marginBottom: '20px'
          }}>
            <h3 style={{ fontSize: '1.3rem', marginBottom: '10px' }}>
              Server-Side Rendering Benefits
            </h3>
            <p style={{ color: '#666', lineHeight: '1.6' }}>
              SSR provides better SEO, faster initial page loads, and improved performance
              on slower devices. Virust makes it easy to build SSR applications with Rust.
            </p>
            <div style={{ marginTop: '15px', fontSize: '0.9rem', color: '#888' }}>
              Posted on March 5, 2026
            </div>
          </article>
        </section>

        <section style={{
          padding: '30px',
          background: '#f0f7ff',
          borderRadius: '8px',
          textAlign: 'center'
        }}>
          <h3 style={{ fontSize: '1.4rem', marginBottom: '10px' }}>
            Start Building Your Blog
          </h3>
          <p style={{ color: '#666', marginBottom: '15px' }}>
            Add your blog posts in src/api/route.rs and create components in web/components/
          </p>
        </section>
      </main>

      <footer style={{
        marginTop: '60px',
        paddingTop: '20px',
        borderTop: '1px solid #eaeaea',
        textAlign: 'center',
        color: '#888'
      }}>
        <p>© 2026 My Blog. Built with Virust.</p>
      </footer>
    </div>
  );
}
"#;
    fs::write(project_dir.join("web/components/HomePage.jsx"), home_page_jsx)?;

    // Create web/index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My Blog - Virust SSR</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: #fafafa;
        }
    </style>
</head>
<body>
    <div id="root">{{SSR_CONTENT}}</div>
    <script type="module" src="/main.js"></script>
</body>
</html>
"#;
    fs::write(project_dir.join("web/index.html"), index_html)?;

    // Create web/main.js
    let main_js = r#"console.log('Virust SSR blog initialized');

// The page is already rendered on the server
// You can add client-side interactivity here
"#;
    fs::write(project_dir.join("web/main.js"), main_js)?;

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
