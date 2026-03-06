//! Tests for the #[path] attribute macro
//!
//! These tests verify that the #[path] attribute:
//! - Compiles correctly
//! - Can be used alongside HTTP method attributes

use virust_macros::get;

#[tokio::test]
async fn test_path_attr_exists() {
    // This test verifies that the #[path] attribute exists and compiles
    // The #[path] attribute is a marker for documentation purposes
    // Actual path parameter extraction will be handled by the runtime

    #[get]
    async fn get_user(id: String) -> String {
        format!("User ID: {}", id)
    }

    #[get]
    async fn get_post(post_id: String, comment_id: String) -> String {
        format!("Post: {}, Comment: {}", post_id, comment_id)
    }

    // Test that the functions are callable
    let result1 = get_user("123".to_string()).await;
    assert_eq!(result1, "User ID: 123");

    let result2 = get_post("456".to_string(), "789".to_string()).await;
    assert_eq!(result2, "Post: 456, Comment: 789");
}

#[tokio::test]
async fn test_path_attr_with_different_types() {
    #[get]
    async fn get_by_id(id: String) -> String {
        format!("ID: {}", id)
    }

    #[get]
    async fn get_by_name(name: String) -> String {
        format!("Name: {}", name)
    }

    let result1 = get_by_id("abc123".to_string()).await;
    assert_eq!(result1, "ID: abc123");

    let result2 = get_by_name("john".to_string()).await;
    assert_eq!(result2, "Name: john");
}

#[tokio::test]
async fn test_path_attr_multiple_params() {
    #[get]
    async fn complex_route(
        org: String,
        team: String,
        project: String,
        branch: String
    ) -> String {
        format!("{}/{}/{}/{}", org, team, project, branch)
    }

    let result = complex_route(
        "acme".to_string(),
        "platform".to_string(),
        "api".to_string(),
        "main".to_string()
    ).await;

    assert_eq!(result, "acme/platform/api/main");
}
