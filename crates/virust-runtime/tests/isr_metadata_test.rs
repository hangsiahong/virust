use virust_runtime::isr::{IsrMetadata, RouteMeta};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_isr_metadata_full_workflow() {
    // Create a temporary directory for test files
    let dir = tempdir().expect("Failed to create temp directory");
    let metadata_path = dir.path().join("isr_metadata.json");

    // 1. Create metadata instance
    let mut metadata = IsrMetadata::new();
    assert_eq!(metadata.routes.len(), 0, "New metadata should have no routes");

    // 2. Add a route with various configurations
    let route = RouteMeta::new("/blog/my-first-post", "web/blog/my-first-post.html", Some(3600))
        .with_tag("blog")
        .with_tag("featured");

    metadata.add_route(route);

    // Add another route without revalidation (static)
    let static_route = RouteMeta::new("/about", "web/about.html", None)
        .with_tag("static");
    metadata.add_route(static_route);

    assert_eq!(metadata.routes.len(), 2, "Should have 2 routes");

    // 3. Save to file
    metadata.save(&metadata_path)
        .expect("Failed to save metadata");

    assert!(metadata_path.exists(), "Metadata file should exist after save");

    // 4. Load back from file
    let loaded_metadata = IsrMetadata::load(&metadata_path)
        .expect("Failed to load metadata");

    // 5. Verify data is preserved
    assert_eq!(loaded_metadata.routes.len(), 2, "Loaded metadata should have 2 routes");

    // Verify first route
    let blog_route = loaded_metadata.get_route("/blog/my-first-post")
        .expect("Blog route should exist");
    assert_eq!(blog_route.path, "/blog/my-first-post");
    assert_eq!(blog_route.file_path, "web/blog/my-first-post.html");
    assert_eq!(blog_route.revalidate, Some(3600));
    assert_eq!(blog_route.tags.len(), 2);
    assert!(blog_route.tags.contains(&"blog".to_string()));
    assert!(blog_route.tags.contains(&"featured".to_string()));

    // Verify second route
    let about_route = loaded_metadata.get_route("/about")
        .expect("About route should exist");
    assert_eq!(about_route.path, "/about");
    assert_eq!(about_route.file_path, "web/about.html");
    assert_eq!(about_route.revalidate, None);
    assert_eq!(about_route.tags.len(), 1);
    assert!(about_route.tags.contains(&"static".to_string()));

    // 6. Test query methods
    let blog_routes = loaded_metadata.routes_by_tag("blog");
    assert_eq!(blog_routes.len(), 1, "Should find 1 blog route");

    let static_routes = loaded_metadata.routes_by_tag("static");
    assert_eq!(static_routes.len(), 1, "Should find 1 static route");

    // 7. Verify JSON format is readable
    let json_content = fs::read_to_string(&metadata_path)
        .expect("Failed to read metadata file");
    assert!(json_content.contains("/blog/my-first-post"), "JSON should contain route path");
    assert!(json_content.contains("blog"), "JSON should contain tag");
    assert!(json_content.contains("featured"), "JSON should contain tag");

    // 8. Test updating metadata
    let mut updated_metadata = loaded_metadata;
    let new_route = RouteMeta::new("/contact", "web/contact.html", Some(7200))
        .with_tag("static");
    updated_metadata.add_route(new_route);

    assert_eq!(updated_metadata.routes.len(), 3, "Should have 3 routes after update");

    // Save updated version
    updated_metadata.save(&metadata_path)
        .expect("Failed to save updated metadata");

    // Load and verify update persisted
    let reloaded_metadata = IsrMetadata::load(&metadata_path)
        .expect("Failed to reload metadata");
    assert_eq!(reloaded_metadata.routes.len(), 3, "Reloaded metadata should have 3 routes");

    let contact_route = reloaded_metadata.get_route("/contact")
        .expect("Contact route should exist");
    assert_eq!(contact_route.revalidate, Some(7200));
}

#[test]
fn test_route_meta_revalidation_check() {
    // Test route with revalidation time
    let route_with_revalidation = RouteMeta::new("/test", "test.html", Some(3600));
    assert!(!route_with_revalidation.needs_revalidation(),
            "Freshly created route should not need revalidation");

    // Test route without revalidation (static)
    let static_route = RouteMeta::new("/static", "static.html", None);
    assert!(!static_route.needs_revalidation(),
            "Static route should never need revalidation");
}

#[test]
fn test_isr_metadata_default() {
    let metadata = IsrMetadata::default();
    assert_eq!(metadata.routes.len(), 0);
}

#[test]
fn test_multiple_routes_with_same_path() {
    let mut metadata = IsrMetadata::new();

    // Add first route
    metadata.add_route(RouteMeta::new("/test", "test1.html", Some(3600)));
    assert_eq!(metadata.routes.len(), 1);

    // Add second route with same path (should overwrite)
    metadata.add_route(RouteMeta::new("/test", "test2.html", Some(7200)));
    assert_eq!(metadata.routes.len(), 1, "Should still have 1 route");

    // Verify it was overwritten
    let route = metadata.get_route("/test").unwrap();
    assert_eq!(route.file_path, "test2.html");
    assert_eq!(route.revalidate, Some(7200));
}

#[test]
fn test_empty_metadata_file() {
    let dir = tempdir().expect("Failed to create temp directory");
    let empty_path = dir.path().join("empty.json");

    // Try to load non-existent file
    let result = IsrMetadata::load(&empty_path);
    assert!(result.is_err(), "Loading non-existent file should fail");

    // Create and save empty metadata
    let empty_metadata = IsrMetadata::new();
    empty_metadata.save(&empty_path).expect("Failed to save empty metadata");

    // Load empty metadata
    let loaded = IsrMetadata::load(&empty_path).expect("Failed to load empty metadata");
    assert_eq!(loaded.routes.len(), 0);
}

#[test]
fn test_route_meta_builder_pattern() {
    let route = RouteMeta::new("/builder", "builder.html", Some(1800))
        .with_tag("api")
        .with_tag("v1")
        .with_tag("cached");

    assert_eq!(route.tags.len(), 3);
    assert!(route.tags.contains(&"api".to_string()));
    assert!(route.tags.contains(&"v1".to_string()));
    assert!(route.tags.contains(&"cached".to_string()));
}
