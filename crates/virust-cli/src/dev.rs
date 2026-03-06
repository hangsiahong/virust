use anyhow::Result;

pub async fn execute() -> Result<()> {
    println!("🚀 Starting Virust development server...");
    println!("Compiling and running your project...");

    // Run the user's project with cargo
    // The user's main.rs handles route registration via api::register_routes()
    let mut child = tokio::process::Command::new("cargo")
        .args(["run"])
        .spawn()?;

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("Cargo run failed with status: {}", status);
    }

    Ok(())
}