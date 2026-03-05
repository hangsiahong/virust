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
