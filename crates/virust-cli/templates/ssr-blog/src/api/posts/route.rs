use virust_macros::{get, ssg};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content: String,
    pub author: String,
    pub published_at: i64,
    pub tags: Vec<String>,
}

// Sample blog posts data
lazy_static::lazy_static! {
    static ref POSTS: Arc<RwLock<Vec<BlogPost>>> = Arc::new(RwLock::new(
        vec![
            BlogPost {
                id: "1".to_string(),
                title: "Getting Started with Virust".to_string(),
                slug: "getting-started-with-virust".to_string(),
                excerpt: "Learn how to build modern web applications with Rust and Virust.".to_string(),
                content: r#"
# Getting Started with Virust

Virust is a modern web framework that combines Rust's performance with React's developer experience.

## Key Features

- **Static Site Generation (SSG)**: Pre-render pages at build time
- **Incremental Static Regeneration (ISR)**: Update static content after build
- **Server-Side Rendering (SSR)**: Dynamic rendering on each request
- **TypeScript Support**: Full type safety in your frontend code
- **Tailwind CSS**: Utility-first CSS framework

## Installation

\`\`\`bash
cargo install virust
virust init my-blog
cd my-blog
cargo run
\`\`\`

## Next Steps

Explore the documentation to learn more about building with Virust!
"#.to_string(),
                author: "Virust Team".to_string(),
                published_at: 1704067200, // 2024-01-01
                tags: vec!["tutorial".to_string(), "getting-started".to_string()],
            },
            BlogPost {
                id: "2".to_string(),
                title: "Understanding SSG and ISR".to_string(),
                slug: "understanding-ssg-and-isr".to_string(),
                excerpt: "Deep dive into Static Site Generation and Incremental Static Regeneration.".to_string(),
                content: r#"
# Understanding SSG and ISR

## Static Site Generation (SSG)

SSG generates HTML at build time, providing:
- Lightning-fast page loads
- SEO optimization
- Reduced server load

## Incremental Static Regeneration (ISR)

ISR allows you to update static pages after build:
- Set revalidation intervals
- Update content without full rebuilds
- Best of both worlds

## Usage in Virust

\`\`\`rust
#[ssg]
async fn page() -> String {
    // Generated at build time
}
\`\`\`

\`\`\`rust
#[ssg(revalidate = 3600)] // 1 hour
async fn page() -> String {
    // Generated at build time, revalidated every hour
}
\`\`\`
"#.to_string(),
                author: "Virust Team".to_string(),
                published_at: 1704153600, // 2024-01-02
                tags: vec!["ssg".to_string(), "isr".to_string(), "performance".to_string()],
            },
            BlogPost {
                id: "3".to_string(),
                title: "Building with TypeScript and Tailwind".to_string(),
                slug: "building-with-typescript-and-tailwind".to_string(),
                excerpt: "Modern frontend development with Virust using TypeScript and Tailwind CSS.".to_string(),
                content: r#"
# Building with TypeScript and Tailwind

Virust provides excellent support for modern frontend development.

## TypeScript Setup

All `.ts` and `.tsx` files are automatically compiled with TypeScript support.

## Tailwind CSS

Tailwind is configured out of the box:

\`\`\`tsx
export default function BlogCard({ post }: { post: BlogPost }) {
  return (
    <div className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition">
      <h2 className="text-2xl font-bold text-gray-900">{post.title}</h2>
      <p className="text-gray-600 mt-2">{post.excerpt}</p>
    </div>
  );
}
\`\`\`

## React Server Components

Virust supports React Server Components for efficient data fetching:

\`\`\`tsx
async function BlogList() {
  const posts = await fetch('/api/posts').then(r => r.json());
  return <div>{posts.map(post => <BlogCard key={post.id} post={post} />)}</div>;
}
\`\`\`
"#.to_string(),
                author: "Virust Team".to_string(),
                published_at: 1704240000, // 2024-01-03
                tags: vec!["typescript".to_string(), "tailwind".to_string(), "react".to_string()],
            },
        ]
    ));
}

/// Get all blog posts (SSG - generated at build time)
#[get]
#[ssg]
async fn route() -> String {
    let posts = POSTS.read().await;
    serde_json::to_string(&*posts).unwrap_or_else(|_| "[]".to_string())
}
