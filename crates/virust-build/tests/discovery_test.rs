use virust_build::discover_ssg_routes;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_discover_ssg_routes() {
    // Create test api directory
    let test_dir = PathBuf::from("/tmp/test_ssg_discovery");
    let _ = fs::remove_dir_all(&test_dir);

    // Create api directory structure
    let api_dir = test_dir.join("api");
    fs::create_dir_all(&api_dir).unwrap();

    // Create test route file
    let blog_dir = api_dir.join("blog/[slug]");
    fs::create_dir_all(&blog_dir).unwrap();

    fs::write(
        blog_dir.join("route.rs"),
        r#"
use virust_macros::{get, ssg};

#[get]
#[ssg(revalidate = 60)]
pub async fn get_blog_post() -> String {
    "test".to_string()
}
"#,
    ).unwrap();

    // Discover routes from api directory
    let routes = discover_ssg_routes(&api_dir).unwrap();

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].path, "/blog/:slug");
    assert_eq!(routes[0].handler, "get_blog_post");
    assert_eq!(routes[0].revalidate, Some(60));

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_discover_multiple_routes() {
    // Create test api directory
    let test_dir = PathBuf::from("/tmp/test_ssg_multiple");
    let _ = fs::remove_dir_all(&test_dir);

    // Create api directory structure
    let api_dir = test_dir.join("api");
    fs::create_dir_all(&api_dir).unwrap();

    // Create home route
    fs::create_dir_all(api_dir.join("home")).unwrap();
    fs::write(
        api_dir.join("home/route.rs"),
        r#"
#[ssg]
pub async fn get_home() -> String {
    "home".to_string()
}
"#,
    ).unwrap();

    // Create blog route with dynamic segment
    let blog_dir = api_dir.join("blog/[id]");
    fs::create_dir_all(&blog_dir).unwrap();
    fs::write(
        blog_dir.join("route.rs"),
        r#"
#[ssg(revalidate = 3600)]
pub async fn get_blog() -> String {
    "blog".to_string()
}
"#,
    ).unwrap();

    // Discover routes
    let routes = discover_ssg_routes(&api_dir).unwrap();

    assert_eq!(routes.len(), 2);

    // Check home route
    let home = routes.iter().find(|r| r.path == "/home").unwrap();
    assert_eq!(home.handler, "get_home");
    assert_eq!(home.revalidate, None);

    // Check blog route
    let blog = routes.iter().find(|r| r.path == "/blog/:id").unwrap();
    assert_eq!(blog.handler, "get_blog");
    assert_eq!(blog.revalidate, Some(3600));

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_path_to_route_conversion() {
    // Test various path patterns
    let test_dir = PathBuf::from("/tmp/test_path_conversion");
    let _ = fs::remove_dir_all(&test_dir);

    // Create api directory structure
    let api_dir = test_dir.join("api");
    fs::create_dir_all(&api_dir).unwrap();

    // Static route
    fs::create_dir_all(api_dir.join("about")).unwrap();
    fs::write(
        api_dir.join("about/route.rs"),
        r#"
#[ssg]
pub async fn about() -> String {
    "about".to_string()
}
"#,
    ).unwrap();

    // Nested dynamic route
    let nested = api_dir.join("users/[userId]/posts/[postId]");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        nested.join("route.rs"),
        r#"
#[ssg]
pub async fn get_post() -> String {
    "post".to_string()
}
"#,
    ).unwrap();

    let routes = discover_ssg_routes(&api_dir).unwrap();

    assert_eq!(routes.len(), 2);

    let about = routes.iter().find(|r| r.path == "/about").unwrap();
    assert_eq!(about.handler, "about");

    let post = routes.iter().find(|r| r.path == "/users/:userId/posts/:postId").unwrap();
    assert_eq!(post.handler, "get_post");

    // Cleanup
    let _ = fs::remove_dir_all(&test_dir);
}
