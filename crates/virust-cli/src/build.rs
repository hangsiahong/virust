use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use std::path::Path;
use virust_build::{SsgBuilder, discover_ssg_routes};

pub struct BuildCommand {
    pub mode: BuildMode,
    pub output: Option<String>,
    pub jobs: Option<usize>,
    pub release: bool,
}

pub enum BuildMode {
    Cargo,
    Ssg,
}

impl BuildCommand {
    pub fn execute(&self) -> Result<()> {
        match &self.mode {
            BuildMode::Cargo => self.build_cargo(),
            BuildMode::Ssg => self.build_ssg(),
        }
    }

    fn build_cargo(&self) -> Result<()> {
        println!("🔨 Building Virust project...");

        if !Path::new("Cargo.toml").exists() {
            anyhow::bail!("Not a Virust project (Cargo.toml not found)");
        }

        let mut args = vec!["build"];
        if self.release {
            args.push("--release");
        }

        let status = Command::new("cargo").args(&args).status()?;

        if status.success() {
            println!("✓ Build complete");
            if self.release {
                println!("  Binary: target/release/virust-server");
            } else {
                println!("  Binary: target/debug/virust-server");
            }
            Ok(())
        } else {
            anyhow::bail!("Build failed");
        }
    }

    fn build_ssg(&self) -> Result<()> {
        println!("🔨 Building static site...");

        // Check if api directory exists
        let api_dir = Path::new("api");
        if !api_dir.exists() {
            anyhow::bail!("API directory not found (expected 'api/' directory)");
        }

        // Determine output directory
        let output_dir = self.output.clone().unwrap_or_else(|| "dist".to_string());
        let output_path = PathBuf::from(&output_dir);

        // Discover routes
        println!("🔍 Discovering SSG routes in api/...");
        let routes = discover_ssg_routes(api_dir)?;

        if routes.is_empty() {
            println!("⚠ No SSG routes found");
            return Ok(());
        }

        println!("✓ Found {} SSG routes:", routes.len());
        for route in &routes {
            let revalidate = route.revalidate.map(|n| format!(" (ISR: {}s)", n)).unwrap_or_default();
            println!("  - {}{}", route.path, revalidate);
        }

        // Create builder
        let parallel_jobs = self.jobs.unwrap_or_else(num_cpus::get);
        println!("⚙ Building with {} parallel job(s)", parallel_jobs);

        let mut builder = SsgBuilder::new(output_path);
        builder.routes = routes;
        builder.parallel_jobs = parallel_jobs;

        // Build
        println!("🚀 Starting build...");
        let stats = tokio::runtime::Runtime::new()?.block_on(builder.build())?;

        // Print results
        println!("\n✓ Build complete!");
        println!("  Pages built: {}", stats.pages_built);
        if stats.pages_failed > 0 {
            println!("  Pages failed: {}", stats.pages_failed);
        }
        if stats.routes_with_isr > 0 {
            println!("  ISR routes: {}", stats.routes_with_isr);
        }
        println!("  Build time: {}ms", stats.build_time_ms);
        println!("  Output: {}/", output_dir);

        Ok(())
    }
}

// Legacy function for backward compatibility
pub fn execute(release: bool) -> Result<()> {
    let cmd = BuildCommand {
        mode: BuildMode::Cargo,
        output: None,
        jobs: None,
        release,
    };
    cmd.execute()
}