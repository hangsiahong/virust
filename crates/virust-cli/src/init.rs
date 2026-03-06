use anyhow::Result;
use std::fs;
use std::path::Path;
use std::env;

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Extract crate name from path (basename)
    let crate_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    // Check if user has VIRUST_PATH set (for development)
    let use_path_deps = std::env::var("VIRUST_PATH").is_ok();

    // Create project structure
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("src/api"))?;
    fs::create_dir_all(project_dir.join("web"))?;

    // Create Cargo.toml with git dependencies (or path for development)
    let (dependencies, build_dependencies) = if use_path_deps {
        // Development mode: use path dependencies
        let virust_path = std::env::var("VIRUST_PATH").unwrap();
        let deps = format!(
            r#"[dependencies]
virust-runtime = {{ path = "{}/crates/virust-runtime" }}
virust-macros = {{ path = "{}/crates/virust-macros" }}
virust-protocol = {{ path = "{}/crates/virust-protocol" }}"#,
            virust_path, virust_path, virust_path
        );
        let build_deps = format!(
            r#"[build-dependencies]
virust-build = {{ path = "{}/crates/virust-build" }}"#,
            virust_path
        );
        (deps, build_deps)
    } else {
        // Production mode: use git dependencies
        (
            r#"[dependencies]
virust-runtime = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }
virust-macros = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }
virust-protocol = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }"#.to_string(),
            r#"[build-dependencies]
virust-build = { git = "https://github.com/hangsiahong/virust.git", branch = "master" }"#.to_string()
        )
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

{}"#,
        crate_name, dependencies, build_dependencies
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
        project_name = crate_name.replace("-", "_")
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

    // Show helpful next steps based on template
    if template == "fullstack-todo" {
        println!("🚀 Quick Start Guide:");
        println!("  1. cd {}", name);
        println!("  2. cargo run        # Build and start the server");
        println!("  3. Open http://localhost:3000");
        println!();
        println!("   Features:");
        println!("   • ✅ Server-Side Rendering (SSR) with Rust + Bun");
        println!("   • ✅ File-based routing (/api/todos)");
        println!("   • ✅ In-memory todo storage");
        println!("   • ✅ React + Tailwind CSS (CDN-based)");
        println!();
        println!("   API Endpoints:");
        println!("   • GET    /api/todos       - List all todos");
        println!("   • POST   /api/todos       - Create todo");
        println!("   • GET    /api/todos/:id   - Get todo details");
        println!("   • PUT    /api/todos/:id   - Update todo");
        println!("   • DELETE /api/todos/:id   - Delete todo");
        println!();
        println!("   Development:");
        println!("   • Edit src/api/todos/route.rs for backend logic");
        println!("   • Edit web/main.js for frontend changes");
        println!("   • Edit web/styles.css for styling");
        println!();
        println!("   Ready to build your full-stack app! 🎉");
    } else {
        println!("Next steps:");
        println!("  cd {}", name);
        println!("  virust dev  # Start server on http://127.0.0.1:3000");
    }

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
    // Create .virust directory and copy Bun renderer files
    fs::create_dir_all(project_dir.join(".virust"))?;

    // Include the bundled renderer files
    fs::write(project_dir.join(".virust/renderer.js"), include_str!("../../virust-bun/bundled/renderer.js"))?;
    fs::write(project_dir.join(".virust/client.js"), include_str!("../../virust-bun/bundled/client.js"))?;
    fs::write(project_dir.join(".virust/package.json"), include_str!("../../virust-bun/bundled/package.json"))?;

    // Also create package.json in project root for JSX resolution
    // When Bun loads JSX files from web/components, it looks for deps from project root
    let root_package_json = r#"{
  "name": "virust-ssr-project",
  "version": "0.1.0",
  "type": "module",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
"#;
    fs::write(project_dir.join("package.json"), root_package_json)?;

    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::get;

    // Register blog routes with SSG enabled
    //
    // These routes use Static Site Generation (SSG):
    // - /blog: Uses ISR with 1-hour revalidation (see route.rs)
    // - /about: Fully static, no revalidation (see route.rs)
    //
    // To build static files:
    //   virust build --ssg
    //
    // SSG vs SSR:
    // - SSR (Server-Side Rendering): HTML generated on each request
    // - SSG (Static Site Generation): HTML pre-generated at build time
    // - ISR (Incremental Static Regeneration): Static pages with periodic revalidation
    //
    // The #[ssg] attributes on these routes enable static site generation.
    // Check src/api/route.rs to see the #[ssg] and #[ssg(revalidate = N)] attributes.
    router
        .route("/blog", get(route::home))
        .route("/about", get(route::about))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with SSR and SSG implementation
    let route_rs = r##"use axum::response::Html;
use virust_macros::{get, ssg};
use virust_runtime::RenderedHtml;

/// Home page with ISR (Incremental Static Regeneration)
///
/// This page uses SSG with a 1-hour revalidation time.
/// - Static HTML is generated at build time: `virust build --ssg`
/// - At runtime, the page is served from the static cache
/// - After 1 hour, the page is regenerated on next request
///
/// Benefits: Fast initial load, excellent SEO, periodic content updates
#[get]
#[ssg(revalidate = 3600)]
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

/// About page with static generation
///
/// This page is fully static with no revalidation.
/// - Static HTML is generated once at build time: `virust build --ssg`
/// - The page is served from the static cache
/// - No regeneration unless you rebuild
///
/// Benefits: Instant loading, perfect for rarely-changing content
#[get]
#[ssg]
pub async fn about() -> Html<String> {
    let rendered = RenderedHtml::new("AboutPage");

    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => {
            eprintln!("SSR Error: {}", e);
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
export default function HomePage() {
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

    // Create AboutPage.jsx component
    let about_page_jsx = r#"// AboutPage.jsx - Static page component
export default function AboutPage() {
  // This component is statically generated at build time
  // Use #[ssg] attribute in Rust to enable static generation

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
          About This Blog
        </h1>
        <p style={{ color: '#666', fontSize: '1.1rem' }}>
          Built with Virust SSG
        </p>
      </header>

      <main>
        <section style={{ marginBottom: '40px' }}>
          <h2 style={{ fontSize: '1.8rem', marginBottom: '20px' }}>
            What is SSG?
          </h2>
          <p style={{ color: '#666', lineHeight: '1.6', marginBottom: '15px' }}>
            <strong>Static Site Generation (SSG)</strong> pre-renders pages at build time,
            creating static HTML files that can be served instantly.
          </p>
          <p style={{ color: '#666', lineHeight: '1.6', marginBottom: '15px' }}>
            This page is marked with <code>#[ssg]</code> in Rust, which tells Virust to
            generate it statically when you run <code>virust build --ssg</code>.
          </p>
        </section>

        <section style={{ marginBottom: '40px' }}>
          <h2 style={{ fontSize: '1.8rem', marginBottom: '20px' }}>
            Benefits of SSG
          </h2>
          <ul style={{ lineHeight: '1.8', color: '#666', paddingLeft: '20px' }}>
            <li><strong>Lightning Fast:</strong> Pre-rendered HTML serves instantly</li>
            <li><strong>Great SEO:</strong> Search engines can crawl static HTML easily</li>
            <li><strong>Low Server Load:</strong> No need to render pages on each request</li>
            <li><strong>CDN Friendly:</strong> Static files can be distributed globally</li>
          </ul>
        </section>

        <section style={{
          padding: '30px',
          background: '#f0f7ff',
          borderRadius: '8px'
        }}>
          <h3 style={{ fontSize: '1.4rem', marginBottom: '10px' }}>
            SSG vs SSR vs ISR
          </h3>
          <ul style={{ lineHeight: '1.8', color: '#666', paddingLeft: '20px' }}>
            <li><strong>SSG:</strong> Static, generated at build time (this page)</li>
            <li><strong>SSR:</strong> Dynamic, rendered on each request</li>
            <li><strong>ISR:</strong> Static with periodic revalidation (blog home page)</li>
          </ul>
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
    fs::write(project_dir.join("web/components/AboutPage.jsx"), about_page_jsx)?;

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
    // Create .virust directory and copy Bun renderer files
    fs::create_dir_all(project_dir.join(".virust"))?;

    // Include the bundled renderer files
    fs::write(project_dir.join(".virust/renderer.js"), include_str!("../../virust-bun/bundled/renderer.js"))?;
    fs::write(project_dir.join(".virust/client.js"), include_str!("../../virust-bun/bundled/client.js"))?;
    fs::write(project_dir.join(".virust/package.json"), include_str!("../../virust-bun/bundled/package.json"))?;

    // Also create package.json in project root for JSX resolution
    // When Bun loads JSX files from web/components, it looks for deps from project root
    let root_package_json = r#"{
  "name": "virust-ssr-project",
  "version": "0.1.0",
  "type": "module",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
"#;
    fs::write(project_dir.join("package.json"), root_package_json)?;

    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create api/mod.rs
    let api_mod = r#"pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::get;

    // Register dashboard routes with SSG enabled
    //
    // These routes use Static Site Generation (SSG):
    // - /dashboard: Uses ISR with 5-minute revalidation (see route.rs)
    // - /settings: Fully static, no revalidation (see route.rs)
    //
    // To build static files:
    //   virust build --ssg
    //
    // The #[ssg] attributes on these routes enable static site generation.
    // Check src/api/route.rs to see the #[ssg] and #[ssg(revalidate = N)] attributes.
    router
        .route("/dashboard", get(route::dashboard))
        .route("/settings", get(route::settings))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create api/route.rs with SSR and SSG implementation
    let route_rs = r##"use axum::response::Html;
use virust_macros::{get, ssg};
use virust_runtime::RenderedHtml;
use serde_json::json;

/// Dashboard page with ISR (Incremental Static Regeneration)
///
/// This page uses SSG with a 5-minute revalidation time.
/// - Static HTML is generated at build time: `virust build --ssg`
/// - At runtime, the page is served from the static cache
/// - After 5 minutes, stats are refreshed on next request
///
/// Benefits: Fast loading, near-real-time data, reduced server load
#[get]
#[ssg(revalidate = 300)]
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

/// Settings page with static generation
///
/// This page is fully static with no revalidation.
/// - Static HTML is generated once at build time: `virust build --ssg`
/// - The page is served from the static cache
/// - No regeneration unless you rebuild
///
/// Benefits: Instant loading, perfect for settings pages that rarely change
#[get]
#[ssg]
pub async fn settings() -> Html<String> {
    let rendered = RenderedHtml::new("SettingsPage");

    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => {
            eprintln!("SSR Error: {}", e);
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

export default function Dashboard({ title, stats }) {
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

    // Create SettingsPage.jsx component
    let settings_page_jsx = r#"// SettingsPage.jsx - Static settings page component
export default function SettingsPage() {
  // This component is statically generated at build time
  // Use #[ssg] attribute in Rust to enable static generation

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
          Settings
        </h1>
        <p style={{ color: '#666', fontSize: '1.1rem' }}>
          Static page example
        </p>
      </header>

      <main>
        <section style={{
          background: 'white',
          padding: '30px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
          marginBottom: '20px'
        }}>
          <h2 style={{ fontSize: '1.8rem', marginBottom: '20px', color: '#333' }}>
            Static Site Generation
          </h2>
          <p style={{ color: '#666', lineHeight: '1.6', marginBottom: '15px' }}>
            This <strong>Settings</strong> page is marked with <code>#[ssg]</code> in Rust,
            which means it's pre-rendered at build time as static HTML.
          </p>
          <p style={{ color: '#666', lineHeight: '1.6', marginBottom: '15px' }}>
            Static pages are ideal for:
          </p>
          <ul style={{ lineHeight: '1.8', color: '#666', paddingLeft: '20px' }}>
            <li>Settings pages that don't change frequently</li>
            <li>Documentation pages</li>
            <li>Legal pages (terms, privacy)</li>
            <li>Help and support pages</li>
          </ul>
        </section>

        <section style={{
          background: 'white',
          padding: '30px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
        }}>
          <h2 style={{ fontSize: '1.8rem', marginBottom: '20px', color: '#333' }}>
            SSG vs SSR Comparison
          </h2>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
            gap: '20px'
          }}>
            <div style={{
              padding: '20px',
              background: '#f0f7ff',
              borderRadius: '6px',
              border: '1px solid #d0e3ff'
            }}>
              <h3 style={{ fontSize: '1.2rem', marginBottom: '10px', color: '#0066cc' }}>
                This Page (SSG)
              </h3>
              <p style={{ color: '#666', lineHeight: '1.6', fontSize: '0.95rem' }}>
                Pre-rendered at build time. Instant loading, perfect for rarely-changing content.
              </p>
            </div>
            <div style={{
              padding: '20px',
              background: '#fff7e6',
              borderRadius: '6px',
              border: '1px solid #ffd966'
            }}>
              <h3 style={{ fontSize: '1.2rem', marginBottom: '10px', color: '#cc6600' }}>
                Dashboard (ISR)
              </h3>
              <p style={{ color: '#666', lineHeight: '1.6', fontSize: '0.95rem' }}>
                Static with 5-minute revalidation. Best balance of freshness and performance.
              </p>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
"#;
    fs::write(project_dir.join("web/components/SettingsPage.jsx"), settings_page_jsx)?;

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
    // Create .virust directory and copy Bun renderer files
    fs::create_dir_all(project_dir.join(".virust"))?;

    // Include the bundled renderer files
    fs::write(project_dir.join(".virust/renderer.js"), include_str!("../../virust-bun/bundled/renderer.js"))?;
    fs::write(project_dir.join(".virust/client.js"), include_str!("../../virust-bun/bundled/client.js"))?;
    fs::write(project_dir.join(".virust/package.json"), include_str!("../../virust-bun/bundled/package.json"))?;

    // Create package.json in project root for React dependencies
    let root_package_json = r#"{
  "name": "virust-fullstack-todo",
  "version": "0.1.0",
  "type": "module",
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
"#;
    fs::write(project_dir.join("package.json"), root_package_json)?;

    // Create lib.rs
    let lib_rs = r#"pub mod api;
"#;
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create main.rs
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

    // Register user routes from the api module
    let router = crate::api::register_routes(router);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("🚀 Server running on http://127.0.0.1:{}", port);

    axum::serve(listener, router).await?;

    Ok(())
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // Create api directory
    fs::create_dir_all(project_dir.join("src/api"))?;

    // Create api/mod.rs with proper route registration
    let api_mod = r#"pub mod todos;

use axum::Router;

/// Register all API routes with the router
pub fn register_routes(router: Router) -> Router {
    // Simple approach: use the functions directly with Axum's built-in extractors
    //
    // Note: This TODO app uses SSR for all routes because the data is dynamic
    // and user-specific. For static pages, you can add the #[ssg] attribute.
    //
    // Example of adding a static help page:
    // 1. Add a new route in todos/route.rs:
    //    ```rust
    //    #[ssg]
    //    #[get]
    //    pub async fn help_page() -> Html<String> { ... }
    //    ```
    // 2. Register it here:
    //    ```rust
    //    router.route("/help", axum::routing::get(todos::route::help_page))
    //    ```
    // 3. Build static files: `virust build --ssg`
    router
        .route("/api/todos", axum::routing::get(todos::route::get_todos))
        .route("/api/todos", axum::routing::post(todos::route::create_todo))
        .route("/api/todos/list", axum::routing::get(todos::route::list_todos))
        .route("/api/todos/:id", axum::routing::get(todos::id_route::get_todo))
        .route("/api/todos/:id", axum::routing::put(todos::id_route::update_todo))
        .route("/api/todos/:id", axum::routing::delete(todos::id_route::delete_todo))
}
"#;
    fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;

    // Create todos directory
    fs::create_dir_all(project_dir.join("src/api/todos"))?;

    // Create todos/mod.rs
    let todos_mod = r#"pub mod route;

// Dynamic routes: use #[path] to reference [id] directory
#[path = "[id]/route.rs"]
pub mod id_route;

// Export the original functions
pub use route::{get_todos, list_todos, create_todo, TodoResponse, CreateTodoRequest};
pub use id_route::{get_todo, update_todo, delete_todo, UpdateTodoRequest};
"#;
    fs::write(project_dir.join("src/api/todos/mod.rs"), todos_mod)?;

    // Create todos/route.rs with in-memory storage and get_todos endpoint
    let todos_route = r#"use axum::{response::Html, Json};
use virust_macros::{get, post};
use virust_runtime::RenderedHtml;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: u64,
}

#[derive(Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

// In-memory todo storage
type TodoStore = Arc<RwLock<Vec<TodoResponse>>>;

pub fn get_todo_store() -> TodoStore {
    use std::sync::OnceLock;
    static STORE: OnceLock<TodoStore> = OnceLock::new();
    STORE.get_or_init(|| {
        Arc::new(RwLock::new(Vec::new()))
    }).clone()
}

/// Get list of todos as JSON (API endpoint)
#[get]
pub async fn get_todos() -> Json<Vec<TodoResponse>> {
    let store = get_todo_store();
    let todos = store.read().await;
    Json(todos.clone())
}

/// Todo list page with server-side rendering
///
/// SSG vs SSR Decision Guide:
///
/// For this TODO app, we use SSR (not SSG) because:
/// - TODOs change frequently (user creates/updates/deletes)
/// - Content is user-specific and dynamic
/// - We need real-time data fetching
///
/// When to use #[ssg] instead:
/// - Static pages that rarely change (about, settings, documentation)
/// - Public content that can be pre-rendered (blog posts, marketing pages)
/// - Pages with data that updates periodically (use #[ssg(revalidate = N)])
///
/// Example of adding SSG to a static help page:
/// ```rust
/// use virust_macros::ssg;
///
/// #[ssg]  // Pure static
/// #[get]
/// pub async fn help_page() -> Html<String> {
///     // This would be pre-rendered at build time
/// }
/// ```
/// Then run: `virust build --ssg`
#[get]
pub async fn list_todos() -> Html<String> {
    let rendered = RenderedHtml::new("TodoList");

    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => {
            eprintln!("SSR Error: {}", e);
            Html(format!(
                "<!DOCTYPE html>\\n<html>\\n<head><title>Error</title></head>\\n<body>\\n    <h1>SSR Error</h1>\\n    <p>{}</p>\\n</body>\\n</html>",
                e.to_string()
            ))
        }
    }
}

/// Create new todo endpoint
#[post]
pub async fn create_todo(
    Json(input): Json<CreateTodoRequest>,
) -> Json<TodoResponse> {
    let new_todo = TodoResponse {
        id: uuid::Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description,
        completed: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let store = get_todo_store();
    let mut todos = store.write().await;
    todos.push(new_todo.clone());

    Json(new_todo)
}
"#;
    fs::write(project_dir.join("src/api/todos/route.rs"), todos_route)?;

    // Create todos/[id]/route.rs for dynamic routes
    fs::create_dir_all(project_dir.join("src/api/todos/[id]"))?;

    let todo_id_route = r#"use axum::{response::Html, Json, extract::Path as AxumPath};
use virust_macros::{get, put, delete};
use virust_runtime::RenderedHtml;
use serde::Deserialize;

// Re-export types and helper from parent module
pub use crate::api::todos::route::{TodoResponse, get_todo_store};

#[derive(Deserialize)]
pub struct UpdateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

/// Individual todo page with server-side rendering
#[get]
pub async fn get_todo(
    AxumPath(id): AxumPath<String>,
) -> Html<String> {
    let rendered = RenderedHtml::with_props("TodoDetail", serde_json::json!({"id": id}));

    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => {
            eprintln!("SSR Error: {}", e);
            Html(format!(
                "<!DOCTYPE html>\\n<html>\\n<head><title>Error</title></head>\\n<body>\\n    <h1>SSR Error</h1>\\n    <p>{}</p>\\n</body>\\n</html>",
                e.to_string()
            ))
        }
    }
}

/// Update todo endpoint
#[put]
pub async fn update_todo(
    AxumPath(id): AxumPath<String>,
    Json(update): Json<UpdateTodoRequest>,
) -> Json<Option<TodoResponse>> {
    let store = get_todo_store();
    let mut todos = store.write().await;

    if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
        todo.title = update.title;
        todo.description = update.description;
        if let Some(completed) = update.completed {
            todo.completed = completed;
        }
        return Json(Some(todo.clone()));
    } else {
        return Json(None);
    }
}

/// Delete todo endpoint
#[delete]
pub async fn delete_todo(AxumPath(id): AxumPath<String>) -> Json<serde_json::Value> {
    let store = get_todo_store();
    let mut todos = store.write().await;

    let original_len = todos.len();
    todos.retain(|todo| todo.id != id);
    let deleted = todos.len() < original_len;

    Json(serde_json::json!({
        "success": deleted,
        "id": id
    }))
}
"#;
    fs::write(project_dir.join("src/api/todos/[id]/route.rs"), todo_id_route)?;

    // Create web/components directory with sample components
    fs::create_dir_all(project_dir.join("web/components"))?;

    // Create a simple TodoList.tsx component (for SSR demonstration)
    let todo_list = r#"// TodoList.tsx - Server component for displaying todo list
import TodoItem from './TodoItem';
import AddTodoForm from './AddTodoForm';

interface Todo {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
  created_at: number;
}

export default function TodoList(): JSX.Element {
  // This is rendered on the server with data
  const todos: Todo[] = [];

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
    fs::write(project_dir.join("web/components/TodoList.tsx"), todo_list)?;

    // Create a simple TodoItem component
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
    fs::write(project_dir.join("web/components/TodoItem.tsx"), todo_item)?;

    // Create AddTodoForm component
    let add_todo_form = r#"'use client';

import { useState, FormEvent } from 'react';

interface CreateTodoRequest {
  title: string;
  description?: string;
}

interface AddTodoFormProps {
  onAddTodo: (title: string, description: string) => void;
}

export default function AddTodoForm({ onAddTodo }: AddTodoFormProps): JSX.Element {
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
        onAddTodo(title.trim(), description.trim());
        setTitle('');
        setDescription('');
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
    fs::write(project_dir.join("web/components/AddTodoForm.tsx"), add_todo_form)?;

    // Create DeleteButton component
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
    fs::write(project_dir.join("web/components/DeleteButton.tsx"), delete_button)?;

    // Create web/index.html pointing to main.js
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
    <script src="/main.js"></script>
</body>
</html>
"#;
    fs::write(project_dir.join("web/index.html"), index_html)?;

    // Create web/main.js with CDN React (no build step needed)
    let main_js = r#"// Simple client-side rendering using CDN React
console.log('Todo app initializing...');

// The root element where React will mount
const rootElement = document.getElementById('root');

if (!rootElement) {
  console.error('Root element not found!');
} else {
  console.log('Root element found, loading React...');

  // Load React from CDN
  loadScript('https://unpkg.com/react@18/umd/react.production.min.js', () => {
    loadScript('https://unpkg.com/react-dom@18/umd/react-dom.production.min.js', () => {
      loadScript('https://unpkg.com/@babel/standalone/babel.min.js', () => {
        console.log('All scripts loaded, initializing app...');
        initApp();
      });
    });
  });
}

function loadScript(url, callback) {
  const script = document.createElement('script');
  script.src = url;
  script.onload = callback;
  script.onerror = () => console.error(`Failed to load ${url}`);
  document.head.appendChild(script);
}

function initApp() {
  const { useState, useEffect } = React;
  const { createRoot } = ReactDOM;

  // TodoList Component
  function TodoList() {
    const [todos, setTodos] = useState([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
      fetch('/api/todos')
        .then(res => res.json())
        .then(data => {
          setTodos(data);
          setLoading(false);
        })
        .catch(err => {
          console.error('Failed to load todos:', err);
          setLoading(false);
        });
    }, []);

    const handleAddTodo = (title, description) => {
      const newTodo = {
        id: Date.now().toString(),
        title,
        description: description || null,
        completed: false,
        created_at: Date.now() / 1000
      };
      setTodos([...todos, newTodo]);
    };

    const handleToggleTodo = (id) => {
      setTodos(todos.map(todo =>
        todo.id === id ? { ...todo, completed: !todo.completed } : todo
      ));
    };

    const handleDeleteTodo = (id) => {
      setTodos(todos.filter(todo => todo.id !== id));
    };

    return React.createElement('div', { className: 'max-w-3xl mx-auto px-4 py-10 font-sans' },
      React.createElement('header', { className: 'mb-10 text-center' },
        React.createElement('h1', { className: 'text-5xl mb-2 bg-gradient-to-r from-purple-500 to-indigo-600 bg-clip-text text-transparent' }, '📝 Todo App'),
        React.createElement('p', { className: 'text-gray-500 mt-2' }, 'Full-stack SSR with Rust + React')
      ),
      React.createElement(AddTodoForm, { onAddTodo: handleAddTodo }),
      React.createElement('div', { className: 'mt-8' },
        loading ?
          React.createElement('div', { className: 'text-center py-16 bg-gray-50 rounded-lg text-gray-500' },
            React.createElement('p', { className: 'text-xl' }, 'Loading... ⏳')
          ) :
        todos.length === 0 ?
          React.createElement('div', { className: 'text-center py-16 bg-gray-50 rounded-lg text-gray-500' },
            React.createElement('p', { className: 'text-xl' }, 'No todos yet. Create one above! 🚀')
          ) :
          React.createElement('ul', { className: 'space-y-3' },
            todos.map(todo =>
              React.createElement(TodoItem, {
                key: todo.id,
                todo: todo,
                onToggle: handleToggleTodo,
                onDelete: handleDeleteTodo
              })
            )
          )
      ),
      React.createElement('footer', { className: 'mt-16 pt-6 border-t border-gray-200 text-center text-gray-500 text-sm' },
        React.createElement('p', null, 'Built with ❤️ using Virust v0.4'),
        React.createElement('p', { className: 'mt-1' }, '• Server-side rendering • File-based routing • TypeScript + Tailwind')
      )
    );
  }

  // TodoItem Component
  function TodoItem({ todo, onToggle, onDelete }) {
    return React.createElement('li', { className: 'bg-white border border-gray-200 rounded-lg p-4 shadow-sm flex items-center gap-3 transition-all hover:shadow-md' },
      React.createElement('input', {
        type: 'checkbox',
        checked: todo.completed,
        onChange: () => onToggle(todo.id),
        className: 'w-5 h-5 cursor-pointer'
      }),
      React.createElement('div', {
        className: `flex-1 ${todo.completed ? 'line-through text-gray-400' : 'text-gray-800'}`
      },
        React.createElement('div', { className: 'font-semibold text-lg' }, todo.title),
        todo.description && React.createElement('div', { className: 'text-sm text-gray-600 mt-1' }, todo.description)
      ),
      React.createElement(DeleteButton, { todoId: todo.id, onDelete })
    );
  }

  // AddTodoForm Component
  function AddTodoForm({ onAddTodo }) {
    const [title, setTitle] = useState('');
    const [description, setDescription] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);

    const handleSubmit = async (e) => {
      e.preventDefault();
      if (!title.trim()) return;

      setIsSubmitting(true);

      try {
        const response = await fetch('/api/todos', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            title: title.trim(),
            description: description.trim() || undefined
          }),
        });

        if (response.ok) {
          onAddTodo(title.trim(), description.trim());
          setTitle('');
          setDescription('');
        }
      } catch (error) {
        console.error('Failed to create todo:', error);
        alert('Failed to create todo. Please try again.');
      } finally {
        setIsSubmitting(false);
      }
    };

    return React.createElement('form', {
      onSubmit: handleSubmit,
      className: 'bg-white p-6 rounded-lg shadow-md mb-8'
    },
      React.createElement('h2', { className: 'text-2xl mb-4 text-gray-800' }, 'Add New Todo'),
      React.createElement('div', { className: 'flex flex-col gap-3' },
        React.createElement('div', null,
          React.createElement('label', { className: 'block mb-1 font-medium text-gray-700' }, 'Title *'),
          React.createElement('input', {
            type: 'text',
            value: title,
            onChange: (e) => setTitle(e.target.value),
            placeholder: 'What needs to be done?',
            required: true,
            className: 'w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500'
          })
        ),
        React.createElement('div', null,
          React.createElement('label', { className: 'block mb-1 font-medium text-gray-700' }, 'Description (optional)'),
          React.createElement('textarea', {
            value: description,
            onChange: (e) => setDescription(e.target.value),
            placeholder: 'Add more details...',
            rows: 3,
            className: 'w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500 font-mono text-sm'
          })
        ),
        React.createElement('button', {
          type: 'submit',
          disabled: isSubmitting || !title.trim(),
          className: `px-6 py-2 rounded-md font-medium text-white transition-colors ${isSubmitting || !title.trim()
            ? 'bg-gray-300 cursor-not-allowed'
            : 'bg-purple-600 hover:bg-purple-700 cursor-pointer'
          }`
        }, isSubmitting ? 'Adding...' : 'Add Todo')
      )
    );
  }

  // DeleteButton Component
  function DeleteButton({ todoId, onDelete }) {
    const [isDeleting, setIsDeleting] = useState(false);

    const handleDelete = async () => {
      if (!confirm('Are you sure you want to delete this todo?')) return;

      setIsDeleting(true);

      try {
        const response = await fetch(`/api/todos/${todoId}`, {
          method: 'DELETE',
        });

        if (response.ok) {
          onDelete(todoId);
        }
      } catch (error) {
        console.error('Failed to delete todo:', error);
        alert('Failed to delete todo. Please try again.');
      } finally {
        setIsDeleting(false);
      }
    };

    return React.createElement('button', {
      onClick: handleDelete,
      disabled: isDeleting,
      className: `px-4 py-2 rounded-md text-sm font-medium text-white transition-colors ${isDeleting
        ? 'bg-gray-300 cursor-not-allowed'
        : 'bg-red-500 hover:bg-red-600 cursor-pointer'
      }`
    }, isDeleting ? 'Deleting...' : '🗑️ Delete');
  }

  // Mount the app
  try {
    const root = createRoot(rootElement);
    root.render(React.createElement(TodoList));
    console.log('Todo app mounted successfully!');
  } catch (error) {
    console.error('Failed to mount app:', error);
  }
}
"#;
    fs::write(project_dir.join("web/main.js"), main_js)?;

    // Create comprehensive styles.css with actual CSS (not Tailwind directives)
    let styles_css = r#"/* Tailwind-like utility classes */
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background-color: #f9fafb;
}

.max-w-3xl { max-width: 48rem; }
.mx-auto { margin-left: auto; margin-right: auto; }
.px-4 { padding-left: 1rem; padding-right: 1rem; }
.py-10 { padding-top: 2.5rem; padding-bottom: 2.5rem; }
.font-sans { font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif; }
.mb-10 { margin-bottom: 2.5rem; }
.mb-8 { margin-bottom: 2rem; }
.mb-4 { margin-bottom: 1rem; }
.mb-2 { margin-bottom: 0.5rem; }
.mb-1 { margin-bottom: 0.25rem; }
.mt-8 { margin-top: 2rem; }
.mt-16 { margin-top: 4rem; }
.mt-2 { margin-top: 0.5rem; }
.mt-1 { margin-top: 0.25rem; }
.pt-6 { padding-top: 1.5rem; }
.text-center { text-align: center; }
.text-5xl { font-size: 3rem; line-height: 1; }
.text-2xl { font-size: 1.5rem; line-height: 2rem; }
.text-xl { font-size: 1.25rem; line-height: 1.75rem; }
.text-lg { font-size: 1.125rem; line-height: 1.75rem; }
.text-sm { font-size: 0.875rem; line-height: 1.25rem; }
.bg-gradient-to-r { background-image: linear-gradient(to right, var(--tw-gradient-stops)); }
.from-purple-500 { --tw-gradient-from: #a855f7; --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to, rgba(168, 85, 247, 0)); }
.to-indigo-600 { --tw-gradient-to: #4f46e5; }
.bg-clip-text { -webkit-background-clip: text; background-clip: text; }
.text-transparent { color: transparent; }
.text-gray-800 { color: #1f2937; }
.text-gray-700 { color: #374151; }
.text-gray-600 { color: #4b5563; }
.text-gray-500 { color: #6b7280; }
.text-gray-400 { color: #9ca3af; }
.text-white { color: white; }
.bg-white { background-color: white; }
.bg-gray-50 { background-color: #f9fafb; }
.bg-purple-600 { background-color: #9333ea; }
.bg-purple-600:hover { background-color: #7e22ce; }
.bg-red-500 { background-color: #ef4444; }
.bg-red-500:hover { background-color: #dc2626; }
.bg-gray-300 { background-color: #d1d5db; }
.border { border-width: 1px; }
.border-gray-200 { border-color: #e5e7eb; }
.border-t { border-top-width: 1px; border-top-style: solid; }
.rounded-lg { border-radius: 0.5rem; }
.rounded-md { border-radius: 0.375rem; }
.shadow-md { box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06); }
.shadow-sm { box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05); }
.p-4 { padding: 1rem; }
.p-6 { padding: 1.5rem; }
.p-8 { padding: 2rem; }
.flex { display: flex; }
.flex-col { flex-direction: column; }
.flex-1 { flex: 1 1 0%; }
.items-center { align-items: center; }
.gap-3 { gap: 0.75rem; }
.gap-5 { gap: 1.25rem; }
.space-y-3 > * + * { margin-top: 0.75rem; }
.transition-all { transition-property: all; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms; }
.transition-colors { transition-property: color, background-color, border-color; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms; }
.hover\:shadow-md:hover { box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06); }
.line-through { text-decoration: line-through; }
.w-full { width: 100%; }
.w-5 { width: 1.25rem; }
.h-5 { height: 1.25rem; }
.px-3 { padding-left: 0.75rem; padding-right: 0.75rem; }
.py-2 { padding-top: 0.5rem; padding-bottom: 0.5rem; }
.px-6 { padding-left: 1.5rem; padding-right: 1.5rem; }
.py-16 { padding-top: 4rem; padding-bottom: 4rem; }
.rounded-md { border-radius: 0.375rem; }
.font-medium { font-weight: 500; }
.font-semibold { font-weight: 600; }
.border-gray-300 { border-color: #d1d5db; }
.focus\:outline-none:focus { outline: 2px solid transparent; outline-offset: 2px; }
.focus\:ring-2:focus { --tw-ring-offset-shadow: var(--tw-ring-inset) 0 0 0 var(--tw-ring-offset-width) var(--tw-ring-offset-color); --tw-ring-shadow: var(--tw-ring-inset) 0 0 0 calc(2px + var(--tw-ring-offset-width)) var(--tw-ring-color); box-shadow: var(--tw-ring-offset-shadow), var(--tw-ring-shadow); }
.focus\:ring-purple-500:focus { --tw-ring-color: #a855f7; }
.cursor-pointer { cursor: pointer; }
.cursor-not-allowed { cursor: not-allowed; }
.rows-3 { rows: 3; }
.font-mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }
.block { display: block; }
input[type="text"], textarea { width: 100%; padding: 0.5rem 0.75rem; border: 1px solid #d1d5db; border-radius: 0.375rem; font-size: 1rem; }
input[type="text"]:focus, textarea:focus { outline: none; border-color: #a855f7; box-shadow: 0 0 0 2px #a855f7; }
button { font-family: inherit; }
button:disabled { opacity: 0.6; cursor: not-allowed; }
.py-5 { padding-top: 1.25rem; padding-bottom: 1.25rem; }
"#;

    fs::write(project_dir.join("web/styles.css"), styles_css)?;

    // Create CLAUDE.md for AI assistant context
    let claude_md = r#"# Claude Code Context for Virust Project

This is a Virust project - a real-time Rust backend framework with SSR.

## Project Structure

\`\`\`
your-project/
├ api/                    # Backend routes (Rust)
│ ├ todos/
│ │ ├ route.rs           # GET /api/todos, POST /api/todos
│ │ └ [id]/
│ │   └ route.rs         # GET/PUT/DELETE /api/todos/:id
├ web/                    # Frontend files
│ ├ components/          # React components
│ │ ├ TodoList.tsx       # Server component (async data fetching)
│ │ ├ AddTodoForm.tsx    # Client component ('use client')
│ │ └ DeleteButton.tsx   # Client component
│ ├ main.js              # Client entry point
│ ├ index.html           # HTML entry
│ └ styles.css           # Tailwind-like utility classes
├ Cargo.toml             # Rust dependencies
└ src/
  └ main.rs              # Application entry point
\`\`\`

## Key Technologies

- **Rust**: Backend with Axum framework
- **Virust**: SSR framework with Bun runtime
- **React**: UI with Server-Side Rendering
- **Tailwind CSS**: Utility-first styling (via styles.css)

## Route Handler Patterns

### HTTP Routes

\`\`\`rust
use virust_macros::{get, post, put, delete};
use axum::Json;

#[get]
async fn list_todos() -> Json<Vec<TodoResponse>> {
    // Return JSON response
}

#[post]
async fn create_todo(Json(input): Json<CreateTodoRequest>) -> Json<TodoResponse> {
    // Automatic JSON deserialization
    Json(new_todo)
}

#[put]
async fn update_todo(
    AxumPath(id): AxumPath<String>,
    Json(update): Json<UpdateTodoRequest>
) -> Json<TodoResponse> {
    // Path + body parameters
}
\`\`\`

### SSR Routes

\`\`\`rust
use virust_runtime::RenderedHtml;
use axum::response::Html;

#[get]
pub async fn list_todos() -> Html<String> {
    let rendered = RenderedHtml::new("TodoList");
    match rendered.render().await {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<h1>Error: {}</h1>", e))
    }
}
\`\`\`

## Component Patterns

### Server Components (Default)

\`\`\`tsx
// web/components/TodoList.tsx
export default async function TodoList() {
  const todos = await fetch('/api/todos').then(r => r.json());

  return (
    <div className="max-w-3xl mx-auto p-6">
      {todos.map(todo => (
        <div key={todo.id}>{todo.title}</div>
      ))}
    </div>
  );
}
\`\`\`

### Client Components

\`\`\`tsx
// web/components/AddTodoForm.tsx
'use client';

import { useState } from 'react';

export default function AddTodoForm() {
  const [title, setTitle] = useState('');

  const handleSubmit = async () => {
    await fetch('/api/todos', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title })
    });
  };

  return <form onSubmit={handleSubmit}>...</form>;
}
\`\`\`

## State Management

For shared state between routes:

\`\`\`rust
use std::sync::Arc;
use tokio::sync::RwLock;

type TodoStore = Arc<RwLock<Vec<TodoResponse>>>;

pub fn get_todo_store() -> TodoStore {
    use std::sync::OnceLock;
    static STORE: OnceLock<TodoStore> = OnceLock::new();
    STORE.get_or_init(|| {
        Arc::new(RwLock::new(Vec::new()))
    }).clone()
}
\`\`\`

## TypeScript Types

Access auto-generated TypeScript types at:
\`\`\`
http://localhost:3000/api/__types
\`\`\`

## Development Workflow

1. **Start server**: \`cargo run\`
2. **Access at**: \`http://localhost:3000\`
3. **Backend changes**: Auto-recompile and restart
4. **Frontend changes**: Auto-reload

## Common Tasks

### Add a new route:
1. Create \`api/feature/route.rs\`
2. Add handler function with \`#[get]\`, \`#[post]\`, etc.
3. Run \`cargo run\` (auto-discovers routes)

### Add a dynamic route:
1. Create \`api/feature/[id]/route.rs\`
2. Use \`AxumPath(id): AxumPath<String>\` to extract parameter

### Create a new component:
1. Add to \`web/components/ComponentName.tsx\`
2. Use \`'use client'\` for interactive components
3. Import/use in other components or route handlers

## Styling

Use Tailwind utility classes from \`styles.css\`:
- Layout: \`max-w-3xl mx-auto p-6\`
- Flexbox: \`flex items-center gap-3\`
- Colors: \`bg-purple-600 text-white\`
- Spacing: \`mb-4 mt-6 px-4 py-2\`

## Error Handling

SSR errors should be caught and displayed:

\`\`\`rust
let rendered = RenderedHtml::new("Component");
match rendered.render().await {
    Ok(html) => Html(html),
    Err(e) => {
        eprintln!("SSR Error: {}", e);
        Html(format!("<h1>Error: {}</h1>", e))
    }
}
\`\`\`

## Testing

Run tests with: \`cargo test\`
"#;
    fs::write(project_dir.join("CLAUDE.md"), claude_md)?;

    // Create SKILL.md for skill-based workflows
    let skill_md = r#"# Virust Skills and Workflows

This document describes recommended skills and workflows for working with Virust projects.

## Using with Superpowers Skills

When working on a Virust project, these skills are particularly useful:

### 1. superpowers:brainstorming
**Use when:** Planning new features, components, or routes before implementation.

**Example workflow:**
\`\`\`
You: "I want to add user authentication to my todo app"

[brainstorming skill activates]
- Asks clarifying questions about auth method (JWT vs sessions)
- Explores different approaches
- Presents design for approval
- Creates implementation plan
\`\`\`

### 2. superpowers:test-driven-development
**Use when:** Implementing new route handlers, components, or business logic.

**Example workflow:**
\`\`\`
1. Write failing test for route handler
2. Implement minimal handler code
3. Verify test passes
4. Refactor if needed
5. Commit
\`\`\`

**For Rust routes:**
\`\`\`rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_todo() {
        let request = CreateTodoRequest {
            title: "Test Todo".to_string(),
            description: None,
        };
        let response = create_todo(Json(request)).await;
        assert_eq!(response.title, "Test Todo");
    }
}
\`\`\`

### 3. superpowers:systematic-debugging
**Use when:** Encountering SSR errors, route registration issues, or unexpected behavior.

**Example workflow:**
\`\`\`
1. Describe the symptom (e.g., "Page shows Loading...")
2. Gather data: Check browser console, server logs, network requests
3. Form hypothesis: "JavaScript syntax error preventing hydration"
4. Test hypothesis: Validate JavaScript syntax
5. Fix and verify
\`\`\`

### 4. superpowers:writing-plans
**Use when:** Implementing complex features with multiple steps (e.g., adding authentication, database integration).

**Example plan structure:**
\`\`\`markdown
# Add Database Integration

## Task 1: Setup Database Connection
- Add dependencies to Cargo.toml
- Create connection pool module
- Test connection

## Task 2: Create Migrations
- Write migration for todos table
- Test migration

## Task 3: Update Todo Store
- Replace in-memory store with database
- Update all route handlers
- Test all endpoints
\`\`\`

## Common Workflows

### Adding a New Feature

\`\`\`
1. Use brainstorming skill to plan the feature
2. Create implementation plan with writing-plans skill
3. Execute using test-driven-development
4. Debug any issues with systematic-debugging
5. Request code review when complete
\`\`\`

### Fixing a Bug

\`\`\`
1. Use systematic-debugging skill to investigate
2. Form hypothesis about root cause
3. Write test case that reproduces bug
4. Fix the bug
5. Verify test passes
6. Check for similar issues
\`\`\`

### Creating New Routes

\`\`\`
1. Create file: api/feature/route.rs
2. Add handler with #[get], #[post], etc.
3. Add #[cfg(test)] tests for handler
4. Run cargo run (auto-discovers route)
5. Test endpoint with curl or browser
6. Verify TypeScript types at /api/__types
\`\`\`

### Creating Components

\`\`\`
1. Create file: web/components/ComponentName.tsx
2. Add 'use client' directive if interactive
3. Implement component with async/await for data fetching
4. Use Tailwind classes for styling
5. Test in browser
6. Verify SSR rendering
\`\`\`

## Virust-Specific Patterns

### Route Pattern
\`\`\`rust
// api/feature/route.rs
use virust_macros::{get, post};
use axum::Json;
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct CreateRequest {
    pub field: String,
}

#[derive(Serialize)]
pub struct Response {
    pub field: String,
}

#[post]
async fn create(Json(input): Json<CreateRequest>) -> Json<Response> {
    Json(Response { field: input.field })
}

#[get]
async fn list() -> Json<Vec<Response>> {
    Json(vec![])
}
\`\`\`

### SSR Component Pattern
\`\`\`tsx
// Server component (async)
export default async function DataList() {
  const data = await fetch('/api/data').then(r => r.json());

  return (
    <div className="container">
      {data.map(item => (
        <div key={item.id}>{item.name}</div>
      ))}
    </div>
  );
}
\`\`\`

\`\`\`tsx
// Client component (interactive)
'use client';

import { useState } from 'react';

export default function Form() {
  const [value, setValue] = useState('');

  return (
    <form onSubmit={handleSubmit}>
      <input value={value} onChange={e => setValue(e.target.value)} />
    </form>
  );
}
\`\`\`

## Quick Reference

### Start Development
\`\`\`bash
cargo run
\`\`\`

### Run Tests
\`\`\`bash
cargo test
cargo test -- --nocapture  # See print output
\`\`\`

### Check Types
\`\`\`bash
curl http://localhost:3000/api/__types
\`\`\`

### Add Dependencies
\`\`\`toml
# Cargo.toml
[dependencies]
dependency = "version"
\`\`\`

### Create New Route
\`\`\`bash
# Create file
touch api/feature/route.rs

# Add handler
# Cargo run will auto-discover
\`\`\`
"#;
    fs::write(project_dir.join("SKILL.md"), skill_md)?;

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
