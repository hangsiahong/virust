use virust_macros::render_component;
use virust_runtime::RenderedHtml;
use serde_json::json;

/// Blog post data structure
#[derive(Clone, Debug)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub author: String,
    pub date: String,
    pub likes: i32,
}

/// Mock blog posts data
fn get_blog_posts() -> Vec<BlogPost> {
    vec![
        BlogPost {
            id: "1".to_string(),
            title: "Getting Started with Virust SSR".to_string(),
            excerpt: "Learn how to build server-side rendered applications with Virust and Rust.".to_string(),
            content: r#"
<p>Server-side rendering (SSR) is a powerful technique that allows you to render your web applications on the server before sending them to the client. This provides several benefits:</p>

<ul>
    <li><strong>Better SEO</strong>: Search engines can easily crawl your content</li>
    <li><strong>Faster initial page loads</strong>: Users see content immediately</li>
    <li><strong>Improved performance</strong>: Less JavaScript to download and execute</li>
</ul>

<h2>Why Virust?</h2>

<p>Virust makes SSR easy by combining the performance of Rust with the flexibility of React. You can write your backend in Rust and your frontend in React, with seamless integration between the two.</p>

<h2>Getting Started</h2>

<p>Start by creating a new project with the SSR template:</p>

<pre><code>virust init my-blog --template ssr-blog
cd my-blog
virust dev
</code></pre>

<p>That's it! You now have a fully-functional SSR application running.</p>
"#.to_string(),
            author: "Jane Developer".to_string(),
            date: "March 6, 2026".to_string(),
            likes: 42,
        },
        BlogPost {
            id: "2".to_string(),
            title: "Server Components vs Client Components".to_string(),
            excerpt: "Understanding the difference between server and client components in Virust.".to_string(),
            content: r#"
<p>In Virust, you can use both server components and client components in your application. Understanding when to use each is key to building performant applications.</p>

<h2>Server Components</h2>

<p>Server components are rendered on the server and sent as HTML to the client. They:</p>

<ul>
    <li>Can directly access databases and backend resources</li>
    <li>Reduce the amount of JavaScript sent to the client</li>
    <li>Are the default in Virust</li>
</ul>

<h2>Client Components</h2>

<p>Client components are rendered in the browser and can use React hooks. They:</p>

<ul>
    <li>Marked with <code>'use client'</code> at the top of the file</li>
    <li>Can use useState, useEffect, and other React hooks</li>
    <li>Handle user interactions and state</li>
</ul>

<h2>Example: Client Component</h2>

<pre><code>'use client';

import { useState } from 'react';

export default function LikeButton() {
    const [likes, setLikes] = useState(0);

    return (
        <button onClick={() => setLikes(likes + 1)}>
            ♥ {likes} Likes
        </button>
    );
}
</code></pre>
"#.to_string(),
            author: "Bob Architect".to_string(),
            date: "March 5, 2026".to_string(),
            likes: 28,
        },
        BlogPost {
            id: "3".to_string(),
            title: "Building Real-Time Features with Virust".to_string(),
            excerpt: "Add real-time updates to your SSR application using WebSockets.".to_string(),
            content: r#"
<p>Real-time features are essential for modern web applications. Whether you're building a chat app, live dashboard, or collaborative tool, Virust makes it easy to add real-time capabilities.</p>

<h2>WebSocket Support</h2>

<p>Virust includes built-in WebSocket support through the <code>#[ws]</code> macro. This allows you to create real-time bidirectional communication between the client and server.</p>

<h2>Example Usage</h2>

<p>Here's how you can create a simple WebSocket endpoint:</p>

<pre><code>use virust_macros::ws;

#[ws]
pub async fn chat_ws(mut ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        ws.send(msg).await;
    }
}
</code></pre>

<h2>Use Cases</h2>

<ul>
    <li>Live chat applications</li>
    <li>Real-time dashboards</li>
    <li>Collaborative editing</li>
    <li>Live notifications</li>
    <li>Multiplayer games</li>
</ul>
"#.to_string(),
            author: "Charlie Engineer".to_string(),
            date: "March 4, 2026".to_string(),
            likes: 35,
        },
    ]
}

/// Home page with server-side rendering
#[render_component("BlogList")]
pub async fn home() -> RenderedHtml {
    let posts = get_blog_posts();

    let posts_data: Vec<serde_json::Value> = posts.iter().map(|post| {
        json!({
            "id": post.id,
            "title": post.title,
            "excerpt": post.excerpt,
            "author": post.author,
            "date": post.date,
            "likes": post.likes,
        })
    }).collect();

    RenderedHtml::with_props("BlogList", json!({ "posts": posts_data }))
}

/// Individual blog post page - Note: This is a simplified example
/// In production, you'd want to use path parameters and handle routing properly
/// For now, we'll just show the first blog post
#[render_component("BlogPost")]
pub async fn blog_post() -> RenderedHtml {
    let posts = get_blog_posts();

    // For this example, just return the first post
    // In a real app, you'd use path parameters: blog_post(id: String)
    let post = &posts[0];

    let post_data = json!({
        "id": post.id,
        "title": post.title,
        "content": post.content,
        "author": post.author,
        "date": post.date,
        "likes": post.likes,
    });

    RenderedHtml::with_props("BlogPost", post_data)
}
