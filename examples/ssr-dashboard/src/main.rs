use virust_runtime::VirustApp;
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
    let router = ssr_dashboard::api::register_routes(router);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("🚀 Server running on http://127.0.0.1:{}", port);
    println!("📊 SSR Dashboard Example - Real-time metrics with server-side rendering!");

    axum::serve(listener, router).await?;

    Ok(())
}
