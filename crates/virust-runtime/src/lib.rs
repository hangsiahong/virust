pub mod http;
pub mod websocket;
pub mod watcher;

pub use watcher::create_watcher;

use axum::Router;

pub struct VirustApp {
    http_router: Router,
}

impl VirustApp {
    pub fn new() -> Self {
        Self {
            http_router: Router::new(),
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
