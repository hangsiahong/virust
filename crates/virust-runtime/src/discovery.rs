use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use anyhow::Result;
use walkdir::WalkDir;

pub struct RouteFile {
    pub path: String,
    pub file_path: PathBuf,
}

#[derive(Debug)]
pub struct DiscoveredRoute {
    pub path: String,        // e.g., "/api/chat"
    pub module_path: String, // e.g., "api::chat::route"
}

pub fn discover_routes(api_dir: &Path) -> Result<Vec<RouteFile>, std::io::Error> {
    let mut route_files = Vec::new();

    // Walk api/ directory for route.rs files
    for entry in WalkDir::new(api_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process route.rs files
        if path.file_name() == Some(std::ffi::OsStr::new("route.rs")) {
            // Extract URL path from file path
            let url_path = path.strip_prefix(api_dir)
                .unwrap()
                .to_str()
                .unwrap()
                .trim_start_matches('/');

            // Convert to URL path: api/chat/route.rs → /api/chat
            let normalized_path = url_path.replace("/route.rs", "");

            route_files.push(RouteFile {
                path: format!("/{}", normalized_path),
                file_path: path.to_path_buf(),
            });
        }
    }

    Ok(route_files)
}

pub fn compile_routes(route_files: &[RouteFile]) -> Result<(), Box<dyn std::error::Error>> {
    // Compile as test harness to trigger macro registration
    for route_file in route_files {
        let crate_dir = route_file.file_path.parent()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid file path"))?;

        // Run cargo check --test-harness on the file's directory
        let output = Command::new("cargo")
            .current_dir(crate_dir)
            .args(["check", "--test-harness", "-q"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to compile {}: {}",
                route_file.path,
                stderr
            ).into());
        }
    }

    Ok(())
}

pub fn discover_routes_from_fs(base_dir: &str) -> Result<Vec<DiscoveredRoute>> {
    let mut routes = Vec::new();
    let api_path = Path::new(base_dir);

    if !api_path.exists() {
        return Ok(routes);
    }

    walk_api_directory(&mut routes, api_path, "")?;

    Ok(routes)
}

fn walk_api_directory(routes: &mut Vec<DiscoveredRoute>, dir: &Path, base_path: &str) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check for dynamic route [id]
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            let new_base = if dir_name.starts_with('[') && dir_name.ends_with(']') {
                // Dynamic route
                let param_name = &dir_name[1..dir_name.len()-1];
                format!("{}/:{}", base_path, param_name)
            } else {
                format!("{}/{}", base_path, dir_name)
            };

            walk_api_directory(routes, &path, &new_base)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("route.rs") {
            // Found a route file
            // Build module path from directory structure
            let module_path = if base_path.is_empty() {
                // Route file is at the root level
                "route".to_string()
            } else {
                // Remove leading "/" and replace "/" with "::", then add "::route"
                let path_without_leading = base_path.trim_start_matches('/');
                format!("{}::route", path_without_leading.replace('/', "::"))
            };

            routes.push(DiscoveredRoute {
                path: format!("/api{}", base_path),
                module_path,
            });
        }
    }

    Ok(())
}
