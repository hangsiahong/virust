use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct RouteFile {
    pub path: String,
    pub file_path: PathBuf,
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
