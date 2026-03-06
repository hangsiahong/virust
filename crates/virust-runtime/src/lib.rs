pub mod http;
pub mod websocket;
pub mod watcher;
pub mod registry;
pub mod discovery;
pub mod extraction;
pub mod typescript;
pub mod persistence;
pub mod struct_parser;
pub mod inventory_registry;
pub mod hmr;
pub mod render;
pub mod middleware;
pub mod client;
pub mod watch;
pub use render::RenderedHtml;
pub use middleware::ssr_middleware;
pub use watch::{watch_components, ComponentChange, ChangeType};

use axum::{Router, routing::get};
use tower_http::services::ServeDir;
use http::create_http_router;
use websocket::ws_upgrade;
pub use watcher::create_watcher;
pub use registry::{RouteRegistry, HttpHandler, WsHandler, TypeDefinition, RouteType, RouteEntry, RegisteredRoute, register_type, get_registered_types};
pub use discovery::{discover_routes, discover_routes_from_fs, RouteFile, DiscoveredRoute};
pub use typescript::TypeScriptGenerator;
pub use inventory_registry::{collect_routes};
pub use hmr::HmrWatcher;

use std::path::PathBuf;
use std::sync::OnceLock;
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade, State},
    response::IntoResponse,
};

/// Global BunRenderer instance stored in OnceLock
///
/// This is initialized once by `init_bun_renderer()` and lives for the
/// duration of the program. The OnceLock ensures thread-safe initialization.
static GLOBAL_BUN_RENDERER: OnceLock<virust_bun::BunRenderer> = OnceLock::new();

async fn hmr_websocket_handler(
    ws: WebSocketUpgrade,
    State(hmr): State<HmrWatcher>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| hmr_socket_handler(socket, hmr))
}

/// HMR WebSocket handler with external broadcast channel
///
/// This handler accepts a WebSocket connection and broadcasts
/// component changes from the provided broadcast channel.
async fn hmr_websocket_handler_with_tx(
    ws: WebSocketUpgrade,
    State(hmr_tx): State<tokio::sync::broadcast::Sender<serde_json::Value>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| hmr_socket_handler_with_tx(socket, hmr_tx))
}

async fn hmr_socket_handler(mut socket: WebSocket, hmr: HmrWatcher) {
    let mut rx = hmr.subscribe();

    while rx.recv().await.is_ok() {
        socket.send(axum::extract::ws::Message::Text("reload".into())).await.ok();
    }
}

/// HMR WebSocket socket handler with external broadcast channel
///
/// This handler subscribes to component changes and broadcasts
/// them to connected WebSocket clients.
async fn hmr_socket_handler_with_tx(
    mut socket: WebSocket,
    hmr_tx: tokio::sync::broadcast::Sender<serde_json::Value>,
) {
    let mut rx = hmr_tx.subscribe();

    while let Ok(update) = rx.recv().await {
        socket.send(axum::extract::ws::Message::Text(
            serde_json::to_string(&update).unwrap_or_else(|_| "reload".to_string())
        )).await.ok();
    }
}

pub struct VirustApp {
    http_router: Router,
    registry: RouteRegistry,
    discovered_routes: Vec<DiscoveredRoute>,
    hmr: HmrWatcher,
}

impl VirustApp {
    pub fn new() -> Self {
        let registry = RouteRegistry::new();

        // Discover routes from api/ directory
        let discovered = discovery::discover_routes_from_fs("api")
            .unwrap_or_default();

        let router = create_http_router();

        Self {
            registry,
            discovered_routes: discovered,
            http_router: router,
            hmr: HmrWatcher::new(),
        }
    }

    pub fn router(&self) -> axum::Router {
        self.router_with_auto_registration()
    }

    /// Create a router with HMR WebSocket support
    ///
    /// This method adds a `/__hmr` WebSocket endpoint that broadcasts
    /// component changes to connected clients for hot module replacement.
    ///
    /// # Arguments
    /// * `hmr_tx` - Broadcast channel sender for HMR updates
    ///
    /// # Returns
    /// An Axum router with HMR support
    pub fn router_with_hmr(&self, hmr_tx: tokio::sync::broadcast::Sender<serde_json::Value>) -> axum::Router {
        use axum::routing::get;

        // Serve static files from web/ directory
        let serve_dir = ServeDir::new("web");

        // Start with base router
        let router = axum::Router::new()
            .nest_service("/", serve_dir)
            .route("/__types", get(http::types_handler))
            .route("/__hmr", get(hmr_websocket_handler_with_tx))
            .route("/bun/client.js", get(client::serve_client_script))
            .with_state(hmr_tx);

        router
    }

    /// Automatically register routes from the inventory and return the router
    /// This is used in dev mode to eliminate the need for manual register_routes()
    fn router_with_auto_registration(&self) -> axum::Router {
        use axum::routing::{get, post, put, delete};

        // Serve static files from web/ directory
        let serve_dir = ServeDir::new("web");

        // Start with base router
        let router = axum::Router::new()
            .nest_service("/", serve_dir)
            .route("/__types", get(http::types_handler))
            .route("/__hmr", get(hmr_websocket_handler))
            .route("/ws", get(ws_upgrade))
            .route("/bun/client.js", get(client::serve_client_script))
            .with_state(self.hmr.clone());

        // Note: Automatic route registration from inventory is not yet implemented
        // because Rust doesn't support dynamic function calls by string name.
        // For now, templates should use the register_routes() pattern.
        // Future implementation could use code generation or dynamic loading.

        router
    }
}

impl Default for VirustApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the Bun renderer for SSR
///
/// This attempts to spawn a Bun process for server-side rendering and stores
/// it in a global OnceLock for the duration of the program.
/// If Bun is not available, it returns false but doesn't fail.
///
/// # Arguments
/// * `web_dir` - Path to the web directory containing components
///
/// # Returns
/// * `true` if Bun renderer was successfully initialized
/// * `false` if Bun is not available or initialization failed
///
/// # Note
/// The BunRenderer is stored in a global OnceLock and will live for the
/// duration of the program. This prevents the Bun process from being
/// killed immediately after initialization.
pub async fn init_bun_renderer(web_dir: &PathBuf) -> bool {
    // Try to create a Bun renderer
    match virust_bun::BunRenderer::new() {
        Ok(mut renderer) => {
            // Set the web directory for component discovery
            if let Err(e) = renderer.set_web_dir(web_dir) {
                eprintln!("Warning: Failed to set web directory: {}", e);
                return false;
            }

            // Verify renderer is responsive
            if let Err(e) = renderer.ping().await {
                eprintln!("Warning: Bun renderer not responsive: {}", e);
                return false;
            }

            let count = renderer.component_count();
            if count > 0 {
                println!("📦 Discovered {} component(s)", count);
            }

            // Store the renderer in the global OnceLock
            // This keeps it alive for the duration of the program
            if GLOBAL_BUN_RENDERER.set(renderer).is_err() {
                eprintln!("Warning: BunRenderer already initialized");
                return false;
            }

            true
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize Bun: {}", e);
            false
        }
    }
}

/// Get a reference to the global BunRenderer
///
/// # Returns
/// * `Some(&BunRenderer)` if the renderer was initialized
/// * `None` if the renderer was not initialized or initialization failed
///
/// # Note
/// This returns a reference to the BunRenderer stored in the global OnceLock.
/// The renderer must be initialized via `init_bun_renderer()` before calling
/// this function.
pub fn get_bun_renderer() -> Option<&'static virust_bun::BunRenderer> {
    GLOBAL_BUN_RENDERER.get()
}

#[tokio::test]
async fn test_build_router() {
    let app = VirustApp::new();
    let _router = app.router();

    // Router should be built successfully
    assert!(true);
}
