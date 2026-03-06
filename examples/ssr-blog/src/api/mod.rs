pub mod route;

// This function is called by the runtime to register routes
pub fn register_routes(router: axum::Router) -> axum::Router {
    use axum::routing::get;

    // Register home page route with SSR
    // Note: We call the original function, not the wrapper
    router
        .route("/", get(route::home))
        .route("/post", get(route::blog_post))
}
