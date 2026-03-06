use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

const AVAILABLE_TEMPLATES: &[&str] = &[
    "basic",
    "ssr-blog",
    "ssr-dashboard",
    "fullstack",
    "chat",
    "todo",
];

pub fn execute(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Validate template name
    if !AVAILABLE_TEMPLATES.contains(&template) {
        anyhow::bail!(
            "Unknown template '{}'. Available templates: {}",
            template,
            AVAILABLE_TEMPLATES.join(", ")
        );
    }

    // Extract crate name from path (basename)
    let crate_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    // Check if user has VIRUST_PATH set (for development)
    let use_path_deps = std::env::var("VIRUST_PATH").is_ok();

    // Prepare variables for template replacement
    let crate_name_normalized = crate_name.replace("-", "_");
    let dependencies = get_dependencies(use_path_deps);
    let build_dependencies = get_build_dependencies(use_path_deps);

    let variables = vec![
        ("{{project_name}}", name.to_string()),
        ("{{crate_name}}", crate_name_normalized),
        ("{{dependencies}}", dependencies),
        ("{{build_dependencies}}", build_dependencies),
    ];

    // Copy template directory
    let template_dir = get_template_dir(template)?;
    copy_template_dir(&template_dir, project_dir, &variables)?;

    // Create src/lib.rs if it doesn't exist
    if !project_dir.join("src/lib.rs").exists() {
        let lib_rs = format!(
            r#"pub mod api;

pub use api::register_routes;
"#
        );
        fs::write(project_dir.join("src/lib.rs"), lib_rs)?;
    }

    // Create src/api/mod.rs if it doesn't exist
    if !project_dir.join("src/api/mod.rs").exists() {
        let api_mod = r#"// This module exports API routes
// Add your route modules here

pub fn register_routes(router: axum::Router) -> axum::Router {
    // Register your routes here
    router
}
"#;
        fs::write(project_dir.join("src/api/mod.rs"), api_mod)?;
    }

    println!("✓ Created project '{}'", name);
    println!("✓ Template: {}", template);
    println!();

    // Show helpful next steps
    show_next_steps(name, template);

    Ok(())
}

fn get_template_dir(template: &str) -> Result<PathBuf> {
    // Try current directory first (for development)
    let current_dir = std::env::current_dir()?;
    let current_path = current_dir.join("crates/virust-cli/templates").join(template);

    if current_path.exists() {
        return Ok(current_path);
    }

    // Try to find templates directory relative to executable
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine executable directory"))?;

    let template_path = exe_dir.join("templates").join(template);

    if template_path.exists() {
        return Ok(template_path);
    }

    // Fallback: try development path (for when running from target/debug)
    let dev_path = exe_dir
        .ancestors()
        .nth(2) // Go up from target/debug/virust to project root
        .map(|p| p.join("crates/virust-cli/templates").join(template));

    if let Some(ref path) = dev_path {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(anyhow::anyhow!(
        "Template '{}' not found. Looked in: {:?}, {:?}, {:?}",
        template,
        current_path,
        template_path,
        dev_path
    ))
}

fn copy_template_dir(
    template_dir: &Path,
    project_dir: &Path,
    variables: &[(&str, String)],
) -> Result<()> {
    // Walk through template directory
    let walker = walkdir::WalkDir::new(template_dir).follow_links(true);
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Failed to read directory entry: {}", e);
                continue;
            }
        };
        let path = entry.path();

        // Skip the template directory itself
        if path == template_dir {
            continue;
        }

        // Calculate relative path
        let relative_path = match path.strip_prefix(template_dir) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Warning: Failed to strip prefix for {:?}: {}", path, e);
                continue;
            }
        };

        // Calculate target path
        let target_path = project_dir.join(relative_path);

        // Create directory or copy file
        if path.is_dir() {
            if let Err(e) = fs::create_dir_all(&target_path) {
                eprintln!("Warning: Failed to create directory {:?}: {}", target_path, e);
            }
        } else {
            // Read file content
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Failed to read file {:?}: {}", path, e);
                    continue;
                }
            };

            // Check if file needs variable replacement (.template extension)
            let (final_content, final_target_path) = if path.extension().and_then(|s| s.to_str()) == Some("template") {
                let replaced = replace_variables(content, variables);
                // Remove .template extension properly
                let file_name = path.file_name().unwrap().to_string_lossy();
                let new_name = file_name.replace(".template", "");
                let path_without_template = target_path.with_file_name(&new_name);
                (replaced, path_without_template)
            } else {
                (replace_variables(content, variables), target_path.clone())
            };

            // Ensure parent directory exists
            if let Some(parent) = final_target_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Warning: Failed to create parent directory {:?}: {}", parent, e);
                }
            }

            // Write file
            if let Err(e) = fs::write(&final_target_path, final_content) {
                eprintln!("Warning: Failed to write file {:?}: {}", final_target_path, e);
            }
        }
    }

    Ok(())
}

fn replace_variables(content: String, variables: &[(&str, String)]) -> String {
    let mut result = content;
    for (key, value) in variables {
        result = result.replace(key, value);
    }
    result
}

fn get_dependencies(use_path_deps: bool) -> String {
    if use_path_deps {
        let virust_path = std::env::var("VIRUST_PATH").unwrap();
        format!(
            r#"[dependencies]
virust-runtime = {{ path = "{}/crates/virust-runtime" }}
virust-macros = {{ path = "{}/crates/virust-macros" }}
virust-protocol = {{ path = "{}/crates/virust-protocol" }}"#,
            virust_path, virust_path, virust_path
        )
    } else {
        r#"[dependencies]
virust-runtime = { git = "https://github.com/hangsiahong/virust.git", branch = "feature/v0.5-ssg-caching" }
virust-macros = { git = "https://github.com/hangsiahong/virust.git", branch = "feature/v0.5-ssg-caching" }
virust-protocol = { git = "https://github.com/hangsiahong/virust.git", branch = "feature/v0.5-ssg-caching" }
lazy_static = "1.4"
inventory = "0.3"
axum = "0.7""#.to_string()
    }
}

fn get_build_dependencies(use_path_deps: bool) -> String {
    if use_path_deps {
        let virust_path = std::env::var("VIRUST_PATH").unwrap();
        format!(
            r#"[build-dependencies]
virust-build = {{ path = "{}/crates/virust-build" }}"#,
            virust_path
        )
    } else {
        r#"[build-dependencies]
virust-build = { git = "https://github.com/hangsiahong/virust.git", branch = "feature/v0.5-ssg-caching" }"#.to_string()
    }
}

fn show_next_steps(name: &str, template: &str) {
    match template {
        "basic" => {
            println!("🚀 Quick Start:");
            println!("  cd {}", name);
            println!("  cargo run");
            println!();
            println!("  A minimal API project ready to customize!");
        }
        "ssr-blog" => {
            println!("📝 Blog Template Created!");
            println!("  cd {}", name);
            println!("  npm install    # Install frontend dependencies");
            println!("  cargo run      # Start development server");
            println!();
            println!("  Features:");
            println!("  • SSG for blog home page");
            println!("  • ISR for individual posts (1hr revalidate)");
            println!("  • TypeScript + Tailwind CSS");
            println!("  • React components");
        }
        "ssr-dashboard" => {
            println!("📊 Dashboard Template Created!");
            println!("  cd {}", name);
            println!("  npm install    # Install frontend dependencies");
            println!("  cargo run      # Start development server");
            println!();
            println!("  Features:");
            println!("  • SSG for dashboard pages");
            println!("  • Cached API responses (5min TTL)");
            println!("  • TypeScript + Tailwind CSS");
            println!("  • Chart visualizations");
        }
        "fullstack" => {
            println!("✨ Full-Stack Template Created!");
            println!("  cd {}", name);
            println!("  npm install    # Install frontend dependencies");
            println!("  cargo run      # Start development server");
            println!();
            println!("  All v0.5 Features Demonstrated:");
            println!("  • SSG for marketing pages");
            println!("  • ISR for blog/content");
            println!("  • SSR for user dashboard");
            println!("  • Caching on API routes");
            println!("  • TypeScript + Tailwind + React");
        }
        "chat" => {
            println!("💬 Chat Template Created!");
            println!("  cd {}", name);
            println!("  cargo run");
            println!();
            println!("  Features:");
            println!("  • WebSocket support");
            println!("  • Real-time messaging");
            println!("  • Message history");
        }
        "todo" => {
            println!("✅ Todo Template Created!");
            println!("  cd {}", name);
            println!("  cargo run");
            println!();
            println!("  Features:");
            println!("  • CRUD API for todos");
            println!("  • In-memory storage");
            println!("  • Simple and clean");
        }
        _ => {
            println!("Next steps:");
            println!("  cd {}", name);
            println!("  cargo run");
        }
    }

    println!();
    println!("Available commands:");
    println!("  virust dev     # Start development server");
    println!("  virust build   # Build for production (SSG)");
}
