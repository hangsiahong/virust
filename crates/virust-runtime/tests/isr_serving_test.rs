use virust_runtime::isr::{IsrManager, IsrMetadata, RouteMeta};
use std::path::PathBuf;
use std::time::SystemTime;

#[tokio::test]
async fn test_isr_fresh_page() {
    let output = PathBuf::from("/tmp/test_isr_fresh");
    let _ = std::fs::remove_dir_all(&output);
    std::fs::create_dir_all(&output).unwrap();

    let metadata_path = output.join("isr_metadata.json");

    // Create metadata with fresh page
    let mut metadata = IsrMetadata::new();
    metadata.add_route(RouteMeta {
        path: "/test".to_string(),
        file_path: "test/index.html".to_string(),
        generated_at: SystemTime::now(),
        revalidate: Some(60),
        tags: vec![],
    });
    metadata.save(&metadata_path).unwrap();

    // Create static file
    std::fs::create_dir_all(output.join("test")).unwrap();
    std::fs::write(
        output.join("test/index.html"),
        "<html><body>Test</body></html>"
    ).unwrap();

    let manager = IsrManager::new(output).unwrap();
    let page = manager.get_static_page("/test").await;

    assert!(page.is_some());
    assert!(page.unwrap().0.contains("Test"));
}

#[tokio::test]
async fn test_isr_stale_page() {
    // Test stale page serving - should serve stale but trigger revalidation
    let output = PathBuf::from("/tmp/test_isr_stale");
    let _ = std::fs::remove_dir_all(&output);
    std::fs::create_dir_all(&output).unwrap();

    let metadata_path = output.join("isr_metadata.json");

    // Create metadata with old page (stale)
    let mut metadata = IsrMetadata::new();
    let old_time = SystemTime::now() - std::time::Duration::from_secs(120);
    metadata.add_route(RouteMeta {
        path: "/stale".to_string(),
        file_path: "stale/index.html".to_string(),
        generated_at: old_time,
        revalidate: Some(60), // 60 seconds, so page is stale
        tags: vec![],
    });
    metadata.save(&metadata_path).unwrap();

    // Create static file
    std::fs::create_dir_all(output.join("stale")).unwrap();
    std::fs::write(
        output.join("stale/index.html"),
        "<html><body>Stale</body></html>"
    ).unwrap();

    let manager = IsrManager::new(output).unwrap();
    let page = manager.get_static_page("/stale").await;

    // Should still serve page (stale-while-revalidate)
    assert!(page.is_some());
    assert!(page.unwrap().0.contains("Stale"));
}

#[tokio::test]
async fn test_isr_no_revalidation() {
    // Test page with no revalidation - always serve static
    let output = PathBuf::from("/tmp/test_isr_no_reval");
    let _ = std::fs::remove_dir_all(&output);
    std::fs::create_dir_all(&output).unwrap();

    let metadata_path = output.join("isr_metadata.json");

    // Create metadata with no revalidation
    let mut metadata = IsrMetadata::new();
    metadata.add_route(RouteMeta {
        path: "/static".to_string(),
        file_path: "static/index.html".to_string(),
        generated_at: SystemTime::now(),
        revalidate: None, // No revalidation
        tags: vec![],
    });
    metadata.save(&metadata_path).unwrap();

    // Create static file
    std::fs::create_dir_all(output.join("static")).unwrap();
    std::fs::write(
        output.join("static/index.html"),
        "<html><body>Static</body></html>"
    ).unwrap();

    let manager = IsrManager::new(output).unwrap();
    let page = manager.get_static_page("/static").await;

    assert!(page.is_some());
    assert!(page.unwrap().0.contains("Static"));
}

#[tokio::test]
async fn test_isr_missing_route() {
    // Test request for non-existent route
    let output = PathBuf::from("/tmp/test_isr_missing");
    let _ = std::fs::remove_dir_all(&output);
    std::fs::create_dir_all(&output).unwrap();

    let metadata_path = output.join("isr_metadata.json");

    // Create empty metadata
    let metadata = IsrMetadata::new();
    metadata.save(&metadata_path).unwrap();

    let manager = IsrManager::new(output).unwrap();
    let page = manager.get_static_page("/nonexistent").await;

    assert!(page.is_none());
}
