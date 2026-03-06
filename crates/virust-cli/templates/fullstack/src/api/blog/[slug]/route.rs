use virust_macros::{get, ssg};
use crate::api::blog::BlogPost;

/// Get single blog post (ISR - revalidate every hour)
#[get]
#[ssg(revalidate = 3600)]
async fn route(slug: String) -> String {
    let posts = super::POSTS;
    let post = posts.iter().find(|p| p.slug == slug);

    match post {
        Some(post) => serde_json::to_string(post).unwrap_or_else(|_| "{}".to_string()),
        None => serde_json::json!({"error": "Post not found"}).to_string(),
    }
}
