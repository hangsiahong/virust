use anyhow::Result;
use std::process::Command;
use std::path::Path;

pub fn execute(port: u16) -> Result<()> {
    println!("🚀 Starting Virust dev server on port {}", port);

    // Check if Cargo.toml exists
    if !Path::new("Cargo.toml").exists() {
        anyhow::bail!("Not a Virust project (Cargo.toml not found)");
    }

    // Run cargo watch with build and run
    let status = Command::new("cargo")
        .args(["run", "--"])
        .arg("--port")
        .arg(port.to_string())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Dev server exited with error");
    }
}