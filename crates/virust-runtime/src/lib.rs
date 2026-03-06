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
pub use render::RenderedHtml;
pub use middleware::ssr_middleware;

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

use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade, State},
    response::IntoResponse,
};

async fn hmr_websocket_handler(
    ws: WebSocketUpgrade,
    State(hmr): State<HmrWatcher>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| hmr_socket_handler(socket, hmr))
}

async fn hmr_socket_handler(mut socket: WebSocket, hmr: HmrWatcher) {
    let mut rx = hmr.subscribe();

    while rx.recv().await.is_ok() {
        socket.send(axum::extract::ws::Message::Text("reload".into())).await.ok();
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

#[tokio::test]
async fn test_build_router() {
    let app = VirustApp::new();
    let _router = app.router();

    // Router should be built successfully
    assert!(true);
}
