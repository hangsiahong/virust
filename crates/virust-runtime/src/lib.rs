pub mod http;
pub mod websocket;
pub mod watcher;
pub mod registry;

use axum::{Router, routing::get};
use http::create_http_router;
use websocket::ws_upgrade;
pub use watcher::create_watcher;
pub use registry::{RouteRegistry, HttpHandler, WsHandler, TypeDefinition};

pub struct VirustApp {
    http_router: Router,
}

impl VirustApp {
    pub fn new() -> Self {
        let router = create_http_router();
        Self {
            http_router: router.route("/ws", get(ws_upgrade)),
        }
    }

    pub fn router(&self) -> Router {
        self.http_router.clone()
    }
}

impl Default for VirustApp {
    fn default() -> Self {
        Self::new()
    }
}
