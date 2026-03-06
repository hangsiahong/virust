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

    println!("🚀 Starting {{project_name}} on http://0.0.0.0:{}", port);
    println!("📝 Blog with SSG + ISR enabled");

    let app = VirustApp::new();
    app.serve(port).await?;

    Ok(())
}
