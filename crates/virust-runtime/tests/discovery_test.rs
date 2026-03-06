use virust_runtime::discovery::discover_routes_from_fs;

#[test]
fn test_discover_routes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let api_dir = temp_dir.path().join("api");
    std::fs::create_dir_all(&api_dir).unwrap();

    let routes = discover_routes_from_fs(api_dir.to_str().unwrap()).unwrap();
    assert!(routes.is_empty());
}

#[test]
fn test_discover_routes_with_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let api_dir = temp_dir.path().join("api");
    std::fs::create_dir_all(api_dir.join("chat")).unwrap();

    // Create a test route file
    std::fs::write(
        api_dir.join("chat/route.rs"),
        r#"
pub async fn test_handler() -> &'static str {
    "test"
}
"#
    ).unwrap();

    let routes = discover_routes_from_fs(api_dir.to_str().unwrap()).unwrap();
    assert!(!routes.is_empty());
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].path, "/api/chat");
    assert_eq!(routes[0].module_path, "chat::route");
}

#[test]
fn test_discover_routes_dynamic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let api_dir = temp_dir.path().join("api");
    std::fs::create_dir_all(api_dir.join("users/[id]")).unwrap();

    // Create a test route file
    std::fs::write(
        api_dir.join("users/[id]/route.rs"),
        r#"
pub async fn test_handler() -> &'static str {
    "test"
}
"#
    ).unwrap();

    let routes = discover_routes_from_fs(api_dir.to_str().unwrap()).unwrap();
    assert!(!routes.is_empty());
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].path, "/api/users/:id");
    assert_eq!(routes[0].module_path, "users:::id::route");
}

#[test]
fn test_discover_routes_nonexistent() {
    let routes = discover_routes_from_fs("/nonexistent/path").unwrap();
    assert!(routes.is_empty());
}
