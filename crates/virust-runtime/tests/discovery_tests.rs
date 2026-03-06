use virust_runtime::discovery::discover_routes;

#[tokio::test]
async fn test_discover_empty_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let api_dir = temp_dir.path().join("api");
    std::fs::create_dir_all(&api_dir).unwrap();

    let registry = discover_routes(&api_dir).unwrap();
    assert_eq!(registry.len(), 0);
}

#[tokio::test]
async fn test_discover_registers_routes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let api_dir = temp_dir.path().join("api");
    std::fs::create_dir_all(api_dir.join("chat")).unwrap();

    // Create a test route file
    std::fs::write(
        api_dir.join("chat/route.rs"),
        r#"
use virust_macros::get;

pub async fn test_handler() -> &'static str {
    "test"
}
"#
    ).unwrap();

    let registry = discover_routes(&api_dir).unwrap();
    // Will be implemented to compile and register
    // For now, just verify discovery finds the file
    assert_eq!(registry.len(), 1);
    assert_eq!(registry[0].path, "/chat");
    assert_eq!(registry[0].file_path, api_dir.join("chat/route.rs"));
}
