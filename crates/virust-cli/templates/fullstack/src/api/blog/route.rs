use virust_macros::{get, ssg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub published_at: i64,
}

lazy_static::lazy_static! {
    static ref POSTS: Vec<BlogPost> = vec![
        BlogPost {
            id: "1".to_string(),
            title: "Introduction to Virust".to_string(),
            slug: "introduction-to-virust".to_string(),
            excerpt: "Learn about the modern Rust web framework".to_string(),
            content: "Full content...".to_string(),
            published_at: 1704067200,
        },
        BlogPost {
            id: "2".to_string(),
            title: "SSG vs ISR vs SSR".to_string(),
            slug: "ssg-vs-isr-vs-ssr".to_string(),
            excerpt: "Understanding different rendering strategies".to_string(),
            content: "Full content...".to_string(),
            published_at: 1704153600,
        },
    ];
}

/// Get all blog posts (SSG - generated at build time)
#[get]
#[ssg]
async fn route() -> String {
    serde_json::to_string(&*POSTS).unwrap_or_else(|_| "[]".to_string())
}
