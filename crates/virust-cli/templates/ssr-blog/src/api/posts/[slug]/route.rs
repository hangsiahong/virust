use virust_macros::{get, ssg};
use crate::api::posts::BlogPost;
use std::sync::Arc;
use tokio::sync::RwLock;

// Share the same posts data
lazy_static::lazy_static! {
    static ref POSTS: Arc<RwLock<Vec<BlogPost>>> = {
        // In a real app, this would be shared with the posts route
        // For simplicity, we're defining it here
        Arc::new(RwLock::new(vec![
            BlogPost {
                id: "1".to_string(),
                title: "Getting Started with Virust".to_string(),
                slug: "getting-started-with-virust".to_string(),
                excerpt: "Learn how to build modern web applications with Rust and Virust.".to_string(),
                content: "Full content here...".to_string(),
                author: "Virust Team".to_string(),
                published_at: 1704067200,
                tags: vec!["tutorial".to_string()],
            },
            // ... other posts
        ]))
    };
}

/// Get a single blog post by slug (ISR - revalidate every hour)
#[get]
#[ssg(revalidate = 3600)]
async fn route(slug: String) -> String {
    let posts = POSTS.read().await;
    let post = posts.iter().find(|p| p.slug == slug);

    match post {
        Some(post) => serde_json::to_string(post).unwrap_or_else(|_| "{}".to_string()),
        None => serde_json::json!({"error": "Post not found"}).to_string(),
    }
}
