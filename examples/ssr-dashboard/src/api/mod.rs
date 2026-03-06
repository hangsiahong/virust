pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::get;

    // Register dashboard route with SSR
    router.route("/", get(route::dashboard_wrapper))
}
