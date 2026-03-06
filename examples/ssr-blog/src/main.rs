use virust_runtime::VirustApp;
use std::env;
use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let port = args.iter()
        .position(|x| x == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    // Create base router for SSR (no static file serving, we use our routes)
    let router = Router::new();

    // Register user routes from the api module
    let router = ssr_blog::api::register_routes(router);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("🚀 Server running on http://127.0.0.1:{}", port);
    println!("📝 SSR Blog Example - View source to see server-side rendering in action!");

    axum::serve(listener, router).await?;

    Ok(())
}
