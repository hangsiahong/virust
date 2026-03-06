use crate::{BuildError, Result, SsgRoute};
use std::path::Path;
use walkdir::WalkDir;

/// Discovers all SSG routes in the given api directory.
///
/// This function walks through the api directory recursively, looking for
/// `route.rs` files that contain the `#[ssg]` attribute. It extracts
/// metadata from these files including route path, handler name, and
/// revalidation time.
///
/// # Arguments
/// * `api_dir` - Path to the api directory to search
///
/// # Returns
/// A vector of discovered SSG routes with their metadata
///
/// # Errors
/// Returns an error if the api directory cannot be read or if any
/// route.rs file cannot be read
pub fn discover_ssg_routes(api_dir: &Path) -> Result<Vec<SsgRoute>> {
    let mut routes = Vec::new();

    for entry in WalkDir::new(api_dir)
        .follow_links(true)
        .into_iter()
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Skipping directory entry due to error: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Only process route.rs files
        if !path.file_name().map(|n| n == "route.rs").unwrap_or(false) {
            continue;
        }

        // Read file content with proper error context
        let content = std::fs::read_to_string(path)
            .map_err(|e| BuildError::RouteDiscoveryFailed(
                format!("Failed to read route file {:?}: {}", path, e)
            ))?;

        // Look for #[ssg] attributes
        if let Some(route) = parse_ssg_route(&content, path) {
            routes.push(route);
        }
    }

    Ok(routes)
}

/// Parses a route file to extract SSG metadata.
///
/// This function checks if the file contains the `#[ssg]` attribute and
/// extracts the route path, handler name, and revalidate time.
///
/// # Arguments
/// * `content` - The file content as a string
/// * `path` - The file path (used to derive the route path)
///
/// # Returns
/// Some(SsgRoute) if the file contains #[ssg], None otherwise
fn parse_ssg_route(content: &str, path: &Path) -> Option<SsgRoute> {
    // Check if file contains #[ssg]
    if !content.contains("#[ssg") {
        return None;
    }

    // Extract route path from file path
    // api/blog/[slug]/route.rs -> /blog/:slug
    let route_path = path_to_route(path)?;

    // Extract handler name
    let handler = extract_handler_name(content)?;

    // Extract revalidate time
    let revalidate = extract_revalidate(content);

    Some(SsgRoute {
        path: route_path,
        handler,
        revalidate,
    })
}

/// Converts a file path to a route pattern string.
///
/// This function transforms a file path into a route pattern by:
/// - Stripping the "api" directory prefix
/// - Converting dynamic segments like `[slug]` to `:slug`
/// - Removing the "route.rs" filename
///
/// # Examples
/// - `api/blog/[slug]/route.rs` → `/blog/:slug`
/// - `api/about/route.rs` → `/about`
/// - `api/users/:userId/posts/:postId/route.rs` → `/users/:userId/posts/:postId`
///
/// # Arguments
/// * `path` - The file path to convert
///
/// # Returns
/// Some(route_pattern) if successful, None if the path is invalid
fn path_to_route(path: &Path) -> Option<String> {
    // Convert path to route pattern
    // /api/blog/[slug]/route.rs -> /blog/:slug
    // /home/user/api/blog/[slug]/route.rs -> /blog/:slug

    let mut route = String::new();
    let mut after_api = false;

    for comp in path.components() {
        let comp_str = comp.as_os_str().to_str()?;

        // Start processing after we see "api"
        if comp_str == "api" {
            after_api = true;
            continue;
        }

        // If we haven't found "api" yet, check if this is the last directory
        // before route.rs (for test cases that don't have "api" in path)
        if !after_api {
            continue;
        }

        // Stop at route.rs
        if comp_str == "route.rs" {
            break;
        }

        if comp_str.starts_with('[') && comp_str.ends_with(']') {
            // Dynamic segment [slug] -> :slug
            let param = &comp_str[1..comp_str.len()-1];
            route.push('/');
            route.push(':');
            route.push_str(param);
        } else {
            route.push('/');
            route.push_str(comp_str);
        }
    }

    if route.is_empty() {
        route = "/".to_string();
    }

    Some(route)
}

/// Extracts the handler function name from route file content.
///
/// Looks for the first `pub async fn` declaration and returns the function name.
///
/// # Arguments
/// * `content` - The file content to search
///
/// # Returns
/// Some(function_name) if found, None otherwise
fn extract_handler_name(content: &str) -> Option<String> {
    // Look for pub async fn XXXX
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("pub async fn") {
            let rest = &line["pub async fn".len()..].trim();
            let func_name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            return Some(func_name);
        }
    }
    None
}

/// Extracts the revalidate time from #[ssg] attribute.
///
/// Looks for `#[ssg(revalidate = N)]` and extracts the numeric value N.
///
/// # Arguments
/// * `content` - The file content to search
///
/// # Returns
/// Some(revalidate_time_in_seconds) if found, None otherwise
fn extract_revalidate(content: &str) -> Option<u64> {
    // Look for #[ssg(revalidate = N)]
    for line in content.lines() {
        let line = line.trim();
        if line.contains("#[ssg") && line.contains("revalidate") {
            // Extract number after "revalidate = "
            if let Some(start) = line.find("revalidate = ") {
                let rest = &line[start + "revalidate = ".len()..];
                let num_str: String = rest
                    .chars()
                    .take_while(|c| c.is_numeric())
                    .collect();
                if let Ok(num) = num_str.parse::<u64>() {
                    return Some(num);
                }
            }
        }
    }
    None
}
