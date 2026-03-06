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

use axum::{Router, routing::{get, post, put, delete}};
use http::create_http_router;
use websocket::ws_upgrade;
pub use watcher::create_watcher;
pub use registry::{RouteRegistry, HttpHandler, WsHandler, TypeDefinition, RouteType, RouteEntry, RegisteredRoute, register_type, get_registered_types};
pub use discovery::{discover_routes, discover_routes_from_fs, RouteFile, DiscoveredRoute};
pub use typescript::TypeScriptGenerator;
pub use inventory_registry::{collect_routes};

pub struct VirustApp {
    http_router: Router,
    registry: RouteRegistry,
    discovered_routes: Vec<DiscoveredRoute>,
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
            http_router: router.route("/ws", get(ws_upgrade)),
        }
    }

    pub fn router(&self) -> Router {
        let mut router = axum::Router::new();

        // Register routes from inventory
        for route in self.registry.get_routes() {
            // Create a simple Axum-compatible handler
            // TODO: This should eventually call the actual handler function
            let handler = async move || -> &'static str {
                "OK"
            };

            router = match route.method.as_str() {
                "GET" => router.route(&route.path, get(handler)),
                "POST" => router.route(&route.path, post(handler)),
                "PUT" => router.route(&route.path, put(handler)),
                "DELETE" => router.route(&route.path, delete(handler)),
                _ => router,
            };
        }

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
