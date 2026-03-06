# SSR Blog Example

A complete, working example of a server-side rendered blog built with Virust, demonstrating the power of SSR with Rust and React.

## Features

- **Server-Side Rendering**: HTML is generated on the server for fast page loads and excellent SEO
- **Multiple Blog Posts**: Three example blog posts with rich content
- **Client Components**: Interactive like button with React hooks
- **Routing**: Home page listing and individual blog post pages
- **Modern UI**: Clean, responsive design with gradient accents

## Project Structure

```
ssr-blog/
├── Cargo.toml           # Rust dependencies
├── README.md            # This file
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library exports
│   └── api/
│       ├── mod.rs       # API module exports
│       └── route.rs     # Route handlers with blog data
└── web/
    ├── index.html       # HTML template with SSR placeholder
    ├── main.js          # Client-side JavaScript
    └── components/
        ├── BlogList.jsx    # Server component for home page
        ├── BlogPost.jsx    # Server component for blog posts
        └── LikeButton.jsx  # Client component with hooks
```

## How It Works

### Server-Side Rendering

1. When a user visits `/`, the `home()` function in `src/api/route.rs` is called
2. It fetches blog posts and passes them to the `BlogList` component via `RenderedHtml::with_props()`
3. The component is rendered on the server using Bun
4. The resulting HTML is injected into the `{{SSR_CONTENT}}` placeholder in `index.html`
5. The fully-rendered page is sent to the browser

### Client Components

The `LikeButton` component is marked with `'use client'`, which means:
- It's rendered in the browser, not on the server
- It can use React hooks (useState, useEffect, etc.)
- It handles user interactions (clicking the like button)
- It demonstrates the hybrid architecture of Virust

## Running the Example

### Prerequisites

- Rust installed
- Virust CLI installed
- Bun installed (for JSX rendering)

### Start the Server

```bash
cd examples/ssr-blog
virust dev
```

The server will start on `http://127.0.0.1:3000`

### View in Browser

1. Open `http://127.0.0.1:3000` to see the blog home page
2. Click on any blog post to view the full post
3. Try the like button to see client-side interactivity
4. View the page source to see the server-rendered HTML

## Key Files Explained

### `src/api/route.rs`

Contains the route handlers that fetch data and initiate server-side rendering:

```rust
#[get]
#[render_component("BlogList")]
pub async fn home() -> RenderedHtml {
    let posts = get_blog_posts();
    let posts_data = /* convert to JSON */;
    RenderedHtml::with_props("BlogList", json!({ "posts": posts_data }))
}
```

### `web/components/BlogList.jsx`

A server component that receives props from the backend:

```jsx
export default function BlogList({ posts }) {
    return (
        <div>
            {posts.map(post => (
                <article key={post.id}>{post.title}</article>
            ))}
        </div>
    );
}
```

### `web/components/LikeButton.jsx`

A client component with React hooks:

```jsx
'use client';

import { useState } from 'react';

export default function LikeButton({ initialLikes }) {
    const [likes, setLikes] = useState(initialLikes);
    return <button onClick={() => setLikes(likes + 1)}>♥ {likes}</button>;
}
```

## Customization

### Adding New Blog Posts

Edit the `get_blog_posts()` function in `src/api/route.rs`:

```rust
BlogPost {
    id: "4".to_string(),
    title: "Your New Post".to_string(),
    excerpt: "A brief description...".to_string(),
    content: "<p>Your post content in HTML...</p>".to_string(),
    author: "Your Name".to_string(),
    date: "March 7, 2026".to_string(),
    likes: 0,
}
```

### Changing the UI

Modify the JSX files in `web/components/`:
- `BlogList.jsx` - Home page layout
- `BlogPost.jsx` - Individual post page
- `LikeButton.jsx` - Like button appearance and behavior

### Styling

The components use inline styles for simplicity. For production, consider:
- CSS modules
- Tailwind CSS
- Styled-components
- Or any other React styling solution

## Learning Resources

- [Virust SSR Guide](../../docs/ssr-guide.md) - Complete SSR documentation
- [Template vs Examples](../../docs/ssr-guide.md#templates-vs-examples) - Understanding the difference
- [Client Components](../../docs/ssr-guide.md#client-components) - Using 'use client'

## Next Steps

1. **Explore the code**: Read through the files to understand how SSR works
2. **Make changes**: Modify the UI, add posts, or change the data
3. **Build from scratch**: Use `virust init my-blog --template ssr-blog` to start your own blog
4. **Add features**: Implement comments, search, or category filtering

## License

This example is part of the Virust project and is available under the same license.
