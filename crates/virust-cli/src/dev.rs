use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::signal;

pub async fn execute() -> Result<()> {
    println!("🚀 Starting Virust development server...");

    // Step 0: Install Bun dependencies if package.json exists
    // Install in both project root (for JSX resolution) and .virust (for renderer)
    let project_root = PathBuf::from(".");
    let virust_dir = PathBuf::from(".virust");

    let needs_install = project_root.join("package.json").exists() || virust_dir.join("package.json").exists();

    if needs_install {
        println!("📦 Installing Bun dependencies...");

        // Install in project root for JSX resolution
        if project_root.join("package.json").exists() {
            match tokio::process::Command::new("bun")
                .args(["install"])
                .current_dir(&project_root)
                .status()
                .await
            {
                Ok(status) if status.success() => {
                    println!("✅ Dependencies installed in project root");
                }
                Ok(_) => {
                    println!("⚠️  Bun install failed in project root, but continuing...");
                }
                Err(_) => {
                    println!("ℹ️  Bun not found - skipping dependency installation");
                }
            }
        }

        // Install in .virust for renderer
        if virust_dir.join("package.json").exists() {
            match tokio::process::Command::new("bun")
                .args(["install"])
                .current_dir(&virust_dir)
                .status()
                .await
            {
                Ok(status) if status.success() => {
                    println!("✅ Dependencies installed in .virust");
                }
                Ok(_) => {
                    println!("⚠️  Bun install failed in .virust, but continuing...");
                }
                Err(_) => {
                    println!("ℹ️  Bun not found - skipping dependency installation");
                }
            }
        }
    }

    println!("Compiling and running your project...");

    // Step 1: Initialize Bun SSR renderer
    let web_dir = PathBuf::from("web");
    let ssr_enabled = virust_runtime::init_bun_renderer(&web_dir).await;

    if ssr_enabled {
        println!("✅ SSR enabled with Bun");
    } else {
        println!("⚠️  Bun not available - SSR disabled (server will still work)");
    }

    // Step 2: Set up component watching
    let components_dir = web_dir.join("components");
    let (component_tx, mut component_rx) = mpsc::channel(100);

    // Spawn the component watcher
    if components_dir.exists() {
        let watch_dir = components_dir.clone();
        tokio::spawn(async move {
            virust_runtime::watch_components(watch_dir, component_tx);
        });
        println!("👀 Watching components for changes...");
    } else {
        println!("ℹ️  No components directory found - component watching disabled");
    }

    // Step 3: Spawn component change handler
    let (hmr_tx, _) = tokio::sync::broadcast::channel(100);

    let component_handler = tokio::spawn(async move {
        while let Some(change) = component_rx.recv().await {
            println!("📦 Component changed: {}", change.component_name);

            // TODO: Send invalidation to Bun renderer
            // For now, we just broadcast the HMR update
            let update = serde_json::json!({
                "type": "component-update",
                "component": change.component_name
            });
            let _ = hmr_tx.send(update);
        }
    });

    // Step 4: Set up graceful shutdown
    let shutdown = signal::ctrl_c();

    tokio::pin!(shutdown);

    // Step 5: Run the user's project with cargo
    // The user's main.rs handles route registration via api::register_routes()
    let mut child = tokio::process::Command::new("cargo")
        .args(["run"])
        .spawn()?;

    // Wait for either child process to finish or CTRL+C
    tokio::select! {
        _ = &mut shutdown => {
            println!("\n🛑 Shutting down...");
            let _ = child.start_kill();
            component_handler.abort();
        }
        status = child.wait() => {
            if !status?.success() {
                anyhow::bail!("Cargo run failed");
            }
        }
    }

    Ok(())
}