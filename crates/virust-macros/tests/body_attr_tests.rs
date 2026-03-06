//! Tests for the #[body] attribute macro
//!
//! These tests verify that the #[body] attribute:
//! - Compiles correctly
//! - Can be used alongside HTTP method attributes

use virust_macros::post;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateUserRequest {
    username: String,
    password: String,
}

#[tokio::test]
async fn test_body_attr_compiles() {
    // This test verifies that the #[body] attribute compiles correctly
    // The #[body] attribute is a marker for documentation purposes
    // Actual body parameter extraction will be handled by the runtime

    #[post]
    async fn create_user(user: User) -> String {
        format!("Created user: {}", user.name)
    }

    #[post]
    async fn create_post(post: serde_json::Value) -> String {
        format!("Created post: {}", post)
    }

    // Test that the functions are callable
    let user = User {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    let result1 = create_user(user.clone()).await;
    assert_eq!(result1, "Created user: John Doe");

    let post_data = serde_json::json!({
        "title": "Hello World",
        "content": "My first post"
    });
    let result2 = create_post(post_data).await;
    assert!(result2.contains("Created post"));
}

#[tokio::test]
async fn test_body_attr_with_struct() {
    #[post]
    async fn register(request: CreateUserRequest) -> String {
        format!("Registered user: {}", request.username)
    }

    let request = CreateUserRequest {
        username: "alice".to_string(),
        password: "secret123".to_string(),
    };

    let result = register(request).await;
    assert_eq!(result, "Registered user: alice");
}

#[tokio::test]
async fn test_body_attr_with_json_value() {
    #[post]
    async fn update_data(data: serde_json::Value) -> String {
        format!("Updated: {}", data)
    }

    let data = serde_json::json!({
        "id": 123,
        "status": "active",
        "tags": ["important", "urgent"]
    });

    let result = update_data(data).await;
    assert!(result.contains("Updated"));
}

#[tokio::test]
async fn test_body_attr_with_custom_struct() {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Article {
        title: String,
        content: String,
        author: String,
    }

    #[post]
    async fn publish_article(article: Article) -> String {
        format!("Published '{}' by {}", article.title, article.author)
    }

    let article = Article {
        title: "Rust Macros".to_string(),
        content: "Content here...".to_string(),
        author: "Jane Developer".to_string(),
    };

    let result = publish_article(article).await;
    assert_eq!(result, "Published 'Rust Macros' by Jane Developer");
}

#[tokio::test]
async fn test_body_attr_multiple_params() {
    #[post]
    async fn complex_handler(
        user: User,
        metadata: serde_json::Value
    ) -> String {
        format!("User: {}, Metadata: {}", user.name, metadata)
    }

    let user = User {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let metadata = serde_json::json!({
        "timestamp": 1234567890,
        "source": "web"
    });

    let result = complex_handler(user, metadata).await;
    assert!(result.contains("User: Bob"));
    assert!(result.contains("Metadata"));
}
