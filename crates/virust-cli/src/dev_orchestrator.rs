use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;
use std::fs;

pub struct DevOrchestrator {
    port: u16,
    vite_process: Option<std::process::Child>,
    backend_process: Option<std::process::Child>,
}

impl DevOrchestrator {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            vite_process: None,
            backend_process: None,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        println!("🚀 Starting Virust dev server on port {}", self.port);

        // Check if Cargo.toml exists
        if !Path::new("Cargo.toml").exists() {
            anyhow::bail!("Not a Virust project (Cargo.toml not found)");
        }

        // Check if web directory exists
        if !Path::new("web").exists() {
            anyhow::bail!("web directory not found. Run 'virust init' first.");
        }

        // Check if package.json exists, if not create a basic one
        if !Path::new("web/package.json").exists() {
            self.create_package_json()?;
        }

        // Check if node_modules exists, if not run npm install
        if !Path::new("web/node_modules").exists() {
            println!("📦 Installing frontend dependencies...");
            let install_status = Command::new("npm")
                .current_dir("web")
                .arg("install")
                .status()
                .context("Failed to install npm dependencies")?;

            if !install_status.success() {
                anyhow::bail!("npm install failed");
            }
        }

        // Start Vite dev server in background
        println!("🔧 Starting Vite frontend server...");
        let vite = Command::new("npm")
            .current_dir("web")
            .args(["run", "dev"])
            .spawn()
            .context("Failed to start Vite dev server")?;

        self.vite_process = Some(vite);
        println!("✓ Vite server started");

        // Set up file watcher
        let (tx, rx) = mpsc::channel();

        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            notify::Config::default(),
        )?;

        // Watch api/ and src/ directories
        if Path::new("api").exists() {
            watcher.watch(Path::new("api"), RecursiveMode::Recursive)?;
            println!("👀 Watching api/ for changes");
        }
        if Path::new("src").exists() {
            watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;
            println!("👀 Watching src/ for changes");
        }

        // Start backend server
        self.start_backend()?;

        println!();
        println!("✨ Dev mode is running!");
        println!("   Frontend: http://localhost:5173");
        println!("   Backend:  http://127.0.0.1:{}", self.port);
        println!();
        println!("Press Ctrl+C to stop");
        println!();

        // Main event loop
        loop {
            // Check for file changes
            if let Ok(_event) = rx.recv_timeout(Duration::from_millis(500)) {
                println!();
                println!("📝 File change detected, restarting backend...");
                self.restart_backend()?;
                println!("✓ Backend restarted");
                println!();
            }

            // Check if backend process is still running
            if let Some(ref mut backend) = self.backend_process {
                if let Some(status) = backend.try_wait()? {
                    if !status.success() {
                        anyhow::bail!("Backend process exited with error");
                    }
                    // Backend exited successfully, just restart it
                    self.start_backend()?;
                }
            }
        }
    }

    fn start_backend(&mut self) -> Result<()> {
        // Kill existing backend process if any
        if let Some(mut backend) = self.backend_process.take() {
            let _ = backend.kill();
            let _ = backend.wait();
        }

        // Build and run backend
        let backend = Command::new("cargo")
            .args(["run", "--"])
            .arg("--port")
            .arg(self.port.to_string())
            .spawn()
            .context("Failed to start backend server")?;

        self.backend_process = Some(backend);
        Ok(())
    }

    fn restart_backend(&mut self) -> Result<()> {
        self.start_backend()
    }

    fn create_package_json(&self) -> Result<()> {
        let package_json = r#"{
  "name": "virust-app",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}"#;

        fs::create_dir_all("web")?;
        fs::write("web/package.json", package_json)?;
        println!("✓ Created web/package.json");
        Ok(())
    }
}

impl Drop for DevOrchestrator {
    fn drop(&mut self) {
        println!();
        println!("🛑 Shutting down dev server...");

        // Kill backend process
        if let Some(mut backend) = self.backend_process.take() {
            let _ = backend.kill();
            let _ = backend.wait();
        }

        // Kill Vite process
        if let Some(mut vite) = self.vite_process.take() {
            let _ = vite.kill();
            let _ = vite.wait();
        }

        println!("✓ Dev server stopped");
    }
}
