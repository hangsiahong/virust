use anyhow::Result;

pub async fn execute() -> Result<()> {
    println!("🚀 Starting Virust development server...");

    let app = virust_runtime::VirustApp::new();
    let router = app.router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("✨ Server running on http://127.0.0.1:3000");
    println!("📁 Serving static files from web/");
    println!("📡 API at /api/*");
    println!("🔌 HMR WebSocket at /ws");

    axum::serve(listener, router).await?;

    Ok(())
}