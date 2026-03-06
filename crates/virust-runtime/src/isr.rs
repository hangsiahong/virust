use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use anyhow::Result;

/// Metadata for a single ISR route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMeta {
    /// The URL path for this route (e.g., "/blog/my-post")
    pub path: String,
    /// Path to the generated HTML file
    pub file_path: String,
    /// When this page was generated
    #[serde(with = "serde_system_time")]
    pub generated_at: SystemTime,
    /// Revalidation time in seconds (None = never revalidate)
    pub revalidate: Option<u64>,
    /// Optional tags for grouping routes
    #[serde(default)]
    pub tags: Vec<String>,
}

impl RouteMeta {
    /// Create a new RouteMeta
    pub fn new(
        path: impl Into<String>,
        file_path: impl Into<String>,
        revalidate: Option<u64>,
    ) -> Self {
        Self {
            path: path.into(),
            file_path: file_path.into(),
            generated_at: SystemTime::now(),
            revalidate,
            tags: Vec::new(),
        }
    }

    /// Add a tag to this route
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if this route needs revalidation
    pub fn needs_revalidation(&self) -> bool {
        if let Some(revalidate_secs) = self.revalidate {
            if let Ok(elapsed) = self.generated_at.elapsed() {
                return elapsed.as_secs() >= revalidate_secs;
            }
        }
        false
    }
}

/// ISR metadata container for tracking all static pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsrMetadata {
    /// Map of route path to route metadata
    pub routes: HashMap<String, RouteMeta>,
    /// When this metadata was generated
    #[serde(with = "serde_system_time")]
    pub generated_at: SystemTime,
}

impl IsrMetadata {
    /// Create a new IsrMetadata instance
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            generated_at: SystemTime::now(),
        }
    }

    /// Add a route to the metadata
    pub fn add_route(&mut self, route: RouteMeta) {
        let path = route.path.clone();
        self.routes.insert(path, route);
    }

    /// Save metadata to a JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = std::fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load metadata from a JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let metadata: IsrMetadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    /// Get a route by path
    pub fn get_route(&self, path: &str) -> Option<&RouteMeta> {
        self.routes.get(path)
    }

    /// Get all routes that need revalidation
    pub fn routes_needing_revalidation(&self) -> Vec<&RouteMeta> {
        self.routes
            .values()
            .filter(|route| route.needs_revalidation())
            .collect()
    }

    /// Get all routes with a specific tag
    pub fn routes_by_tag(&self, tag: &str) -> Vec<&RouteMeta> {
        self.routes
            .values()
            .filter(|route| route.tags.contains(&tag.to_string()))
            .collect()
    }
}

impl Default for IsrMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Module for SystemTime serde serialization/deserialization
mod serde_system_time {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::SystemTime;

    // Serialize SystemTime as duration since UNIX epoch
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }

    // Deserialize SystemTime from duration since UNIX epoch
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(secs))
            .ok_or(serde::de::Error::custom("invalid SystemTime value"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_route_meta_creation() {
        let route = RouteMeta::new("/test", "/path/to/test.html", Some(3600));
        assert_eq!(route.path, "/test");
        assert_eq!(route.file_path, "/path/to/test.html");
        assert_eq!(route.revalidate, Some(3600));
        assert!(route.tags.is_empty());
    }

    #[test]
    fn test_route_meta_with_tags() {
        let route = RouteMeta::new("/test", "/path/to/test.html", Some(3600))
            .with_tag("blog")
            .with_tag("featured");
        assert_eq!(route.tags.len(), 2);
        assert!(route.tags.contains(&"blog".to_string()));
    }

    #[test]
    fn test_needs_revalidation() {
        let route = RouteMeta::new("/test", "/path/to/test.html", Some(1));
        // Should not need immediate revalidation
        assert!(!route.needs_revalidation());
    }

    #[test]
    fn test_isr_metadata_new() {
        let metadata = IsrMetadata::new();
        assert!(metadata.routes.is_empty());
    }

    #[test]
    fn test_add_route() {
        let mut metadata = IsrMetadata::new();
        let route = RouteMeta::new("/test", "/path/to/test.html", Some(3600));
        metadata.add_route(route);

        assert_eq!(metadata.routes.len(), 1);
        assert!(metadata.routes.contains_key("/test"));
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("isr_metadata.json");

        // Create metadata and add a route
        let mut metadata = IsrMetadata::new();
        let route = RouteMeta::new("/test", "/path/to/test.html", Some(3600))
            .with_tag("blog");
        metadata.add_route(route);

        // Save to file
        metadata.save(&file_path).unwrap();

        // Load from file
        let loaded = IsrMetadata::load(&file_path).unwrap();

        // Verify data is preserved
        assert_eq!(loaded.routes.len(), 1);
        let loaded_route = loaded.get_route("/test").unwrap();
        assert_eq!(loaded_route.path, "/test");
        assert_eq!(loaded_route.file_path, "/path/to/test.html");
        assert_eq!(loaded_route.revalidate, Some(3600));
        assert_eq!(loaded_route.tags.len(), 1);
        assert_eq!(loaded_route.tags[0], "blog");
    }

    #[test]
    fn test_get_route() {
        let mut metadata = IsrMetadata::new();
        let route = RouteMeta::new("/test", "/path/to/test.html", Some(3600));
        metadata.add_route(route);

        let retrieved = metadata.get_route("/test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().path, "/test");

        let not_found = metadata.get_route("/nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_routes_by_tag() {
        let mut metadata = IsrMetadata::new();

        metadata.add_route(
            RouteMeta::new("/blog1", "/path1.html", Some(3600))
                .with_tag("blog")
        );
        metadata.add_route(
            RouteMeta::new("/blog2", "/path2.html", Some(3600))
                .with_tag("blog")
                .with_tag("featured")
        );
        metadata.add_route(
            RouteMeta::new("/about", "/path3.html", Some(3600))
                .with_tag("static")
        );

        let blog_routes = metadata.routes_by_tag("blog");
        assert_eq!(blog_routes.len(), 2);

        let featured_routes = metadata.routes_by_tag("featured");
        assert_eq!(featured_routes.len(), 1);

        let static_routes = metadata.routes_by_tag("static");
        assert_eq!(static_routes.len(), 1);
    }
}
