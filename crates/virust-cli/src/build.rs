use anyhow::Result;
use std::process::Command;
use std::path::Path;

pub fn execute(release: bool) -> Result<()> {
    println!("🔨 Building Virust project...");

    if !Path::new("Cargo.toml").exists() {
        anyhow::bail!("Not a Virust project (Cargo.toml not found)");
    }

    let mut args = vec!["build"];
    if release {
        args.push("--release");
    }

    let status = Command::new("cargo").args(&args).status()?;

    if status.success() {
        println!("✓ Build complete");
        if release {
            println!("  Binary: target/release/virust-server");
        } else {
            println!("  Binary: target/debug/virust-server");
        }
        Ok(())
    } else {
        anyhow::bail!("Build failed");
    }
}