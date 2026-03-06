use anyhow::Result;
use std::fs;
use std::path::Path;
use std::env;

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Check if user has VIRUST_PATH set (for development)
    let use_path_deps = std::env::var("VIRUST_PATH").is_ok();

    // Create project structure
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("src/api"))?;
    fs::create_dir_all(project_dir.join("web"))?;

    // Create Cargo.toml with git dependencies (or path for development)
    let dependencies = if use_path_deps {
        // Development mode: use path dependencies
        let virust_path = std::env::var("VIRUST_PATH").unwrap();
        format!(
            r#"[dependencies]
virust-runtime = {{ path = "{}/crates/virust-runtime" }}
virust-macros = {{ path = "{}/crates/virust-macros" }}
virust-protocol = {{ path = "{}/crates/virust-protocol" }}"#,
            virust_path, virust_path, virust_path
        )
    } else {
        // Production mode: use git dependencies
        r#"[dependencies]
virust-runtime = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }
virust-macros = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }
virust-protocol = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }"#.to_string()
    };

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

{}
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
        name, dependencies
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Template-specific setup
    match template {
        "chat" => setup_chat_template(project_dir)?,
        "todo" => setup_todo_template(project_dir)?,
        "ssr-blog" => setup_ssr_blog_template(project_dir)?,
        "ssr-dashboard" => setup_ssr_dashboard_template(project_dir)?,
        "fullstack-todo" => setup_fullstack_todo_template(project_dir)?,
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

    // Create minimal web/package.json (no vite dependency) unless template already created one
    if !project_dir.join("web/package.json").exists() {
        let package_json = r#"{
  "name": "virust-app",
  "version": "0.1.0"
}"#;
        fs::write(project_dir.join("web/package.json"), package_json)?;
    }

    println!("✓ Created project '{}'", name);
    println!("✓ Template: {}", template);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  virust dev  # Start server on http://127.0.0.1:3000");
    if use_path_deps {
        println!();
        println!("Note: This project uses path dependencies to the local virust workspace.");
        println!("      Set VIRUST_PATH environment variable if the virust crates are in a custom location.");
    }

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

    // Register blog routes with SSR
    // Use /blog prefix to avoid conflict with /__types route
    router.route("/blog", get(route::home))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with SSR implementation
    let route_rs = r##"use axum::response::Html;
use virust_macros::get;
use virust_runtime::RenderedHtml;

/// Home page with server-side rendering
#[get]
pub async fn home() -> Html<String> {
    let rendered = RenderedHtml::new("HomePage");

    // Render the component to HTML
    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => {
            // Log the error
            eprintln!("SSR Error: {}", e);

            // Return a simple error page
            Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>SSR Error</h1>
    <p>{}</p>
</body>
</html>"#,
                e.to_string()
            ))
        }
    }
}
"##;
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

    // Create package.json for React dependencies (in root so Bun can find them)
    let package_json = r#"{
  "name": "ssr-blog",
  "private": true,
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
"#;
    fs::write(project_dir.join("package.json"), package_json)?;

    Ok(())
}

fn setup_ssr_dashboard_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::get;

    // Register dashboard route with SSR and data passing
    // Use /dashboard prefix to avoid conflict with /__types route
    router.route("/dashboard", get(route::dashboard))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with SSR implementation
    let route_rs = r##"use axum::response::Html;
use virust_macros::get;
use virust_runtime::RenderedHtml;
use serde_json::json;

/// Dashboard page with server-side rendering and data
#[get]
pub async fn dashboard() -> Html<String> {
    // In a real app, you might fetch this data from a database
    let stats = json!({
        "totalUsers": 1250,
        "activeUsers": 342,
        "revenue": 45320,
        "conversionRate": 3.2
    });

    let rendered = RenderedHtml::with_props("Dashboard", stats);

    // Render the component to HTML
    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => {
            // Log the error
            eprintln!("SSR Error: {}", e);

            // Return a simple error page
            Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>SSR Error</h1>
    <p>{}</p>
</body>
</html>"#,
                e.to_string()
            ))
        }
    }
}
"##;
    fs::write(project_dir.join("src/api/route.rs"), route_rs)?;

    // Create web/components directory
    fs::create_dir_all(project_dir.join("web/components"))?;

    // Create Dashboard.jsx component (server component)
    let dashboard_jsx = r#"// Dashboard.jsx - Server-side rendered dashboard with data
import RefreshButton from './RefreshButton';

export default async function Dashboard({ title, stats }) {
  // This component is rendered on the server
  // Data is passed from the backend as props

  return (
    <div style={{
      maxWidth: '1200px',
      margin: '0 auto',
      padding: '20px',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      background: '#f5f5f5',
      minHeight: '100vh'
    }}>
      <header style={{
        marginBottom: '40px',
        paddingBottom: '20px',
        borderBottom: '2px solid #e0e0e0',
        background: 'white',
        padding: '30px',
        borderRadius: '8px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
      }}>
        <h1 style={{
          fontSize: '2.5rem',
          marginBottom: '10px',
          color: '#333'
        }}>
          {title || 'Analytics Dashboard'}
        </h1>
        <p style={{ color: '#666', fontSize: '1.1rem' }}>
          Real-time metrics and insights
        </p>
        <div style={{ marginTop: '20px' }}>
          <RefreshButton />
        </div>
      </header>

      <main>
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
          gap: '20px',
          marginBottom: '40px'
        }}>
          <StatCard
            title="Total Users"
            value={stats?.totalUsers || 0}
            change="+12.5%"
            trend="up"
          />
          <StatCard
            title="Active Users"
            value={stats?.activeUsers || 0}
            change="+8.2%"
            trend="up"
          />
          <StatCard
            title="Revenue"
            value={'$' + ((stats?.revenue || 0) / 1000).toFixed(1) + 'k'}
            change="+23.1%"
            trend="up"
          />
          <StatCard
            title="Conversion Rate"
            value={stats?.conversionRate + '%' || '0%'}
            change="-0.5%"
            trend="down"
          />
        </div>

        <section style={{
          background: 'white',
          padding: '30px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
        }}>
          <h2 style={{ fontSize: '1.8rem', marginBottom: '20px', color: '#333' }}>
            Quick Start Guide
          </h2>
          <ul style={{
            lineHeight: '1.8',
            color: '#666',
            paddingLeft: '20px'
          }}>
            <li>This dashboard uses <strong>server-side rendering</strong> - the HTML is generated on the server</li>
            <li>Data is passed from the backend using <code>RenderedHtml::with_props()</code> in Rust</li>
            <li>The <strong>RefreshButton</strong> is a <em>client component</em> (marked with 'use client')</li>
            <li>Client components can use React hooks and handle user interactions</li>
            <li>Server components can fetch data and pass it as props to client components</li>
          </ul>
        </section>
      </main>
    </div>
  );
}

// Server-side helper component (can be rendered on server)
function StatCard({ title, value, change, trend }) {
  const trendColor = trend === 'up' ? '#10b981' : '#ef4444';
  const trendIcon = trend === 'up' ? '↑' : '↓';

  return (
    <div style={{
      background: 'white',
      padding: '25px',
      borderRadius: '8px',
      boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
      border: '1px solid #e5e7eb'
    }}>
      <h3 style={{
        fontSize: '0.9rem',
        color: '#666',
        marginBottom: '10px',
        textTransform: 'uppercase',
        letterSpacing: '0.5px'
      }}>
        {title}
      </h3>
      <div style={{
        fontSize: '2rem',
        fontWeight: 'bold',
        color: '#111',
        marginBottom: '8px'
      }}>
        {value}
      </div>
      <div style={{
        fontSize: '0.85rem',
        color: trendColor,
        fontWeight: '500'
      }}>
        {trendIcon} {change} from last month
      </div>
    </div>
  );
}
"#;
    fs::write(project_dir.join("web/components/Dashboard.jsx"), dashboard_jsx)?;

    // Create RefreshButton.jsx (client component)
    let refresh_button_jsx = r#"// RefreshButton.jsx - Client-side interactive component
'use client';

import { useState } from 'react';

export default function RefreshButton() {
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [lastRefresh, setLastRefresh] = useState(null);

  const handleRefresh = () => {
    setIsRefreshing(true);

    // Simulate a refresh action
    setTimeout(() => {
      setIsRefreshing(false);
      setLastRefresh(new Date().toLocaleTimeString());
    }, 1000);
  };

  return (
    <div>
      <button
        onClick={handleRefresh}
        disabled={isRefreshing}
        style={{
          padding: '10px 20px',
          fontSize: '1rem',
          fontWeight: '500',
          background: isRefreshing ? '#9ca3af' : '#007bff',
          color: 'white',
          border: 'none',
          borderRadius: '6px',
          cursor: isRefreshing ? 'not-allowed' : 'pointer',
          transition: 'all 0.2s',
          opacity: isRefreshing ? 0.7 : 1
        }}
      >
        {isRefreshing ? '⏳ Refreshing...' : '🔄 Refresh Data'}
      </button>
      {lastRefresh && (
        <div style={{
          marginTop: '10px',
          fontSize: '0.85rem',
          color: '#666'
        }}>
          Last refreshed: {lastRefresh}
        </div>
      )}
    </div>
  );
}
"#;
    fs::write(project_dir.join("web/components/RefreshButton.jsx"), refresh_button_jsx)?;

    // Create web/index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Analytics Dashboard - Virust SSR</title>
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
    let main_js = r#"console.log('Virust SSR dashboard initialized');

// The page is already rendered on the server
// Client components (with 'use client') are hydrated here
"#;
    fs::write(project_dir.join("web/main.js"), main_js)?;

    // Create package.json for React dependencies (in root so Bun can find them)
    let package_json = r#"{
  "name": "ssr-dashboard",
  "private": true,
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
"#;
    fs::write(project_dir.join("package.json"), package_json)?;

    Ok(())
}

fn setup_fullstack_todo_template(project_dir: &Path) -> Result<()> {
    // Create lib.rs with multiple API modules
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs with route registration for all routes
    let api_mod = r#"pub mod todos;
pub mod todos_id;

// Register all routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::{get, post, put, delete};

    router
        // Todo list routes
        .route("/api/todos", get(todos::list_todos))
        .route("/api/todos", post(todos::create_todo))
        // Individual todo routes
        .route("/api/todos/:id", get(todos_id::get_todo))
        .route("/api/todos/:id", put(todos_id::update_todo))
        .route("/api/todos/:id", delete(todos_id::delete_todo))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create todos/list route (SSR)
    let todos_list = r#"use virust_macros::{get, post, render_component};
use virust_macros::body;
use virust_runtime::RenderedHtml;
use serde::Serialize;

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: u64,
}

#[derive(serde::Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

/// Todo list page with server-side rendering
#[get]
#[render_component("TodoList")]
pub async fn list_todos() -> RenderedHtml {
    RenderedHtml::new("TodoList")
}

/// Create new todo endpoint
#[post]
pub async fn create_todo(#[body] input: CreateTodoRequest) -> TodoResponse {
    TodoResponse {
        id: uuid::Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description,
        completed: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}
"#;
    fs::create_dir_all(project_dir.join("src/api/todos"))?;
    fs::write(project_dir.join("src/api/todos/route.rs"), todos_list)?;

    // Create todos_id route (SSR with path parameter)
    let todos_id = r#"use virust_macros::{get, put, delete, render_component};
use virust_macros::path;
use virust_runtime::RenderedHtml;
use serde::Serialize;

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: u64,
}

/// Individual todo page with server-side rendering
#[get]
#[render_component("TodoDetail")]
pub async fn get_todo(#[path] id: String) -> RenderedHtml {
    RenderedHtml::with_props("TodoDetail", serde_json::json!({"id": id}))
}

/// Update todo endpoint
#[put]
pub async fn update_todo(
    #[path] id: String,
    #[body] update: UpdateTodoRequest,
) -> TodoResponse {
    // Update logic would go here
    TodoResponse {
        id: id.clone(),
        title: update.title,
        description: update.description,
        completed: update.completed.unwrap_or(false),
        created_at: 0,
    }
}

#[derive(serde::Deserialize)]
pub struct UpdateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

/// Delete todo endpoint
#[delete]
pub async fn delete_todo(#[path] id: String) -> String {
    // Delete logic would go here
    format!("{{\"success\":true, \"id\":\"{}\"}}", id)
}
"#;
    fs::create_dir_all(project_dir.join("src/api/todos_id"))?;
    fs::write(project_dir.join("src/api/todos_id/route.rs"), todos_id)?;

    // Create web/components directory with TypeScript/TSX components
    fs::create_dir_all(project_dir.join("web/components"))?;

    // TodoList.tsx - Server component with TypeScript and Tailwind
    let todo_list = r#"// TodoList.tsx - Server component for displaying todo list
import TodoItem from './TodoItem';
import AddTodoForm from './AddTodoForm';
import { loadTodos } from './utils';

interface Todo {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
  created_at: number;
}

export default async function TodoList(): Promise<JSX.Element> {
  // This is rendered on the server with data
  const todos: Todo[] = await loadTodos();

  return (
    <div className="max-w-3xl mx-auto px-4 py-10 font-sans">
      <header className="mb-10 text-center">
        <h1 className="text-5xl mb-2 bg-gradient-to-r from-purple-500 to-indigo-600 bg-clip-text text-transparent">
          📝 Todo App
        </h1>
        <p className="text-gray-500 mt-2">
          Full-stack SSR with Rust + React + TypeScript
        </p>
      </header>

      <AddTodoForm />

      <div className="mt-8">
        {todos.length === 0 ? (
          <div className="text-center py-16 bg-gray-50 rounded-lg text-gray-500">
            <p className="text-xl">No todos yet. Create one above! 🚀</p>
          </div>
        ) : (
          <ul className="space-y-3">
            {todos.map((todo: Todo) => (
              <TodoItem key={todo.id} todo={todo} />
            ))}
          </ul>
        )}
      </div>

      <footer className="mt-16 pt-6 border-t border-gray-200 text-center text-gray-500 text-sm">
        <p>Built with ❤️ using Virust v0.4</p>
        <p className="mt-1">
          • Server-side rendering • File-based routing • TypeScript + Tailwind
        </p>
      </footer>
    </div>
  );
}
"#;

    // TodoItem.tsx - Server component with TypeScript
    let todo_item = r#"// TodoItem.tsx - Server component for todo item
import DeleteButton from './DeleteButton';

interface Todo {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
  created_at: number;
}

interface TodoItemProps {
  todo: Todo;
}

export default function TodoItem({ todo }: TodoItemProps): JSX.Element {
  return (
    <li className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm flex items-center gap-3 transition-all hover:shadow-md">
      <input
        type="checkbox"
        checked={todo.completed}
        readOnly
        className="w-5 h-5 cursor-not-allowed"
      />
      <div className={`flex-1 ${todo.completed ? 'line-through text-gray-400' : 'text-gray-800'}`}>
        <div className="font-semibold text-lg">
          {todo.title}
        </div>
        {todo.description && (
          <div className="text-sm text-gray-600 mt-1">
            {todo.description}
          </div>
        )}
      </div>
      <DeleteButton todoId={todo.id} />
    </li>
  );
}
"#;

    // AddTodoForm.tsx - Client component with TypeScript and Tailwind
    let add_todo_form = r#"'use client';

import { useState, FormEvent } from 'react';

interface CreateTodoRequest {
  title: string;
  description?: string;
}

export default function AddTodoForm(): JSX.Element {
  const [title, setTitle] = useState<string>('');
  const [description, setDescription] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>): Promise<void> => {
    e.preventDefault();
    if (!title.trim()) return;

    setIsSubmitting(true);

    try {
      const payload: CreateTodoRequest = {
        title: title.trim(),
        description: description.trim() || undefined,
      };

      const response = await fetch('/api/todos', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (response.ok) {
        setTitle('');
        setDescription('');
        // Reload page to show new todo
        window.location.reload();
      }
    } catch (error) {
      console.error('Failed to create todo:', error);
      alert('Failed to create todo. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded-lg shadow-md mb-8">
      <h2 className="text-2xl mb-4 text-gray-800">
        Add New Todo
      </h2>
      <div className="flex flex-col gap-3">
        <div>
          <label className="block mb-1 font-medium text-gray-700">
            Title *
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="What needs to be done?"
            required
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
          />
        </div>
        <div>
          <label className="block mb-1 font-medium text-gray-700">
            Description (optional)
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Add more details..."
            rows={3}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500 font-mono text-sm"
          />
        </div>
        <button
          type="submit"
          disabled={isSubmitting || !title.trim()}
          className={`px-6 py-2 rounded-md font-medium text-white transition-colors ${
            isSubmitting || !title.trim()
              ? 'bg-gray-300 cursor-not-allowed'
              : 'bg-purple-600 hover:bg-purple-700 cursor-pointer'
          }`}
        >
          {isSubmitting ? 'Adding...' : 'Add Todo'}
        </button>
      </div>
    </form>
  );
}
"#;

    // DeleteButton.tsx - Client component with TypeScript
    let delete_button = r#"'use client';

import { useState } from 'react';

interface DeleteButtonProps {
  todoId: string;
}

export default function DeleteButton({ todoId }: DeleteButtonProps): JSX.Element {
  const [isDeleting, setIsDeleting] = useState<boolean>(false);

  const handleDelete = async (): Promise<void> => {
    if (!confirm('Are you sure you want to delete this todo?')) return;

    setIsDeleting(true);

    try {
      const response = await fetch(`/api/todos/${todoId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        // Reload page to show updated list
        window.location.reload();
      }
    } catch (error) {
      console.error('Failed to delete todo:', error);
      alert('Failed to delete todo. Please try again.');
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <button
      onClick={handleDelete}
      disabled={isDeleting}
      className={`px-4 py-2 rounded-md text-sm font-medium text-white transition-colors ${
        isDeleting
          ? 'bg-gray-300 cursor-not-allowed'
          : 'bg-red-500 hover:bg-red-600 cursor-pointer'
      }`}
    >
      {isDeleting ? 'Deleting...' : '🗑️ Delete'}
    </button>
  );
}
"#;

    // TodoDetail.tsx - Server component with TypeScript
    let todo_detail = r#"// TodoDetail.tsx - Server component for todo detail page
import { loadTodo } from './utils';

interface Todo {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
  created_at: number;
}

interface TodoDetailProps {
  id: string;
}

export default async function TodoDetail({ id }: TodoDetailProps): Promise<JSX.Element> {
  // Fetch todo data on server
  const todo: Todo | null = await loadTodo(id);

  if (!todo) {
    return (
      <div className="px-10 text-center text-gray-500 py-16">
        <h1 className="text-3xl font-bold mb-4">Todo not found</h1>
        <p>The todo you're looking for doesn't exist.</p>
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto px-4 py-10 font-sans">
      <nav className="mb-8">
        <a
          href="/"
          className="text-purple-600 hover:text-purple-700 no-underline font-medium"
        >
          ← Back to Todos
        </a>
      </nav>

      <div className="bg-white p-8 rounded-lg shadow-md">
        <h1 className="text-4xl mb-5 text-gray-800">
          {todo.title}
        </h1>

        {todo.description && (
          <p className="text-lg text-gray-600 leading-relaxed mb-5">
            {todo.description}
          </p>
        )}

        <div className="flex gap-5 py-5 border-t border-gray-200">
          <div>
            <strong>Status:</strong>{' '}
            <span className={todo.completed ? 'text-green-600' : 'text-yellow-600'}>
              {todo.completed ? '✓ Completed' : '○ Pending'}
            </span>
          </div>
          <div>
            <strong>Created:</strong> {new Date(todo.created_at * 1000).toLocaleDateString()}
          </div>
        </div>

        <div className="mt-5 p-5 bg-gray-50 rounded-md">
          <h3 className="text-lg mb-2 text-gray-700">
            Actions
          </h3>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-md text-sm font-medium cursor-pointer transition-colors"
          >
            🔄 Refresh
          </button>
        </div>
      </div>
    </div>
  );
}
"#;

    // utils.ts - Utility functions with TypeScript
    let utils = r#"// utils.ts - Utility functions for data fetching and type definitions

// Type definitions
export interface Todo {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
  created_at: number;
}

// Mock data (replace with actual API calls in production)
export async function loadTodos(): Promise<Todo[]> {
  // In production, fetch from API
  const response = await fetch('/api/todos');
  const todos: Todo[] = await response.json();
  return todos;
}

export async function loadTodo(id: string): Promise<Todo | null> {
  // In production, fetch from API
  const response = await fetch(`/api/todos/${id}`);
  if (!response.ok) return null;
  const todo: Todo = await response.json();
  return todo;
}
"#;

    // Write all component files
    fs::write(project_dir.join("web/components/TodoList.tsx"), todo_list)?;
    fs::write(project_dir.join("web/components/TodoItem.tsx"), todo_item)?;
    fs::write(project_dir.join("web/components/AddTodoForm.tsx"), add_todo_form)?;
    fs::write(project_dir.join("web/components/DeleteButton.tsx"), delete_button)?;
    fs::write(project_dir.join("web/components/TodoDetail.tsx"), todo_detail)?;
    fs::write(project_dir.join("web/components/utils.ts"), utils)?;

    // Create tsconfig.json
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "jsx": "react",
    "strict": true,
    "moduleResolution": "node",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true
  },
  "include": ["components/**/*", "utils.ts"],
  "exclude": ["node_modules"]
}"#;
    fs::write(project_dir.join("web/tsconfig.json"), tsconfig)?;

    // Create tailwind.config.js
    let tailwind_config = r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./components/**/*.{js,ts,jsx,tsx}",
    "./utils.ts"
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}"#;
    fs::write(project_dir.join("web/tailwind.config.js"), tailwind_config)?;

    // Create postcss.config.js
    let postcss_config = r#"module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}"#;
    fs::write(project_dir.join("web/postcss.config.js"), postcss_config)?;

    // Create web/styles.css with Tailwind directives
    let styles_css = r#"@tailwind base;
@tailwind components;
@tailwind utilities;

/* Custom styles if needed */
body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
"#;
    fs::write(project_dir.join("web/styles.css"), styles_css)?;

    // Create web/index.html with Tailwind CDN
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Todo App - Full Stack SSR</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="root">Loading...</div>
    <script type="module" src="/main.ts"></script>
</body>
</html>
"#;
    fs::write(project_dir.join("web/index.html"), index_html)?;

    // Create web/main.ts
    let main_ts = r#"console.log('Todo app initialized with SSR');
console.log('Server components rendered on the server');
console.log('Client components interactive in the browser');
console.log('TypeScript + Tailwind CSS enabled');
"#;
    fs::write(project_dir.join("web/main.ts"), main_ts)?;

    // Create web/package.json with dependencies
    let package_json = r#"{
  "name": "todo-app",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "build": "echo 'No build step required - using Tailwind CDN for development'",
    "typecheck": "tsc --noEmit"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "typescript": "^5.0.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}"#;
    fs::write(project_dir.join("web/package.json"), package_json)?;

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
