# Server-Side Rendering Guide

## Overview

Virust v0.4 supports Next.js-style server-side rendering using Bun as the JavaScript runtime. This enables you to build full-stack applications with React components that render on the server and hydrate in the browser.

### Key Features

- **Server-Side Rendering (SSR)** - Render React components on the server for faster initial page loads
- **React Server Components** - Async components with direct data fetching capabilities
- **Client Components** - Interactive components with React hooks (useState, useEffect, etc.)
- **TypeScript Support** - Full TypeScript/TSX support out of the box
- **Hot Module Replacement** - Instant feedback during development
- **Filesystem Routing** - Automatic route discovery from `api/` directory

## Quick Start

### 1. Create an SSR Application

```bash
virust init my-blog -t ssr-blog
cd my-blog
```

This creates a new project with SSR support, including:
- Rust backend with route handlers
- React components in `web/components/`
- Bun-based SSR runtime
- Example server and client components

### 2. Define a Route with SSR

Create a route handler in your `api/` directory:

```rust
// src/api/route.rs
use virust_macros::{get, render_component};
use virust_runtime::RenderedHtml;

#[get]
#[render_component("HomePage")]
async fn index() -> RenderedHtml {
    RenderedHtml::new("HomePage")
}
```

The `#[render_component]` attribute tells Virust to render the specified React component for this route.

### 3. Create a Server Component

Create a React component in `web/components/`:

```jsx
// web/components/HomePage.jsx
export default async function HomePage() {
  // Server components can use async/await
  const posts = await fetch('http://localhost:3000/api/posts')
    .then(r => r.json());

  return (
    <div>
      <h1>Blog Posts</h1>
      {posts.map(post => (
        <article key={post.id}>
          <h2>{post.title}</h2>
          <p>{post.excerpt}</p>
        </article>
      ))}
    </div>
  );
}
```

### 4. Run Development Server

```bash
virust dev
```

This starts:
- Rust backend server on port 3000
- Bun renderer for SSR
- File watcher for hot module replacement

Visit `http://localhost:3000` to see your SSR application in action.

## Server vs Client Components

Virust supports both server and client components, similar to Next.js 13+.

### Server Components (Default)

Server components run on the server and have the following characteristics:

**Features:**
- Can use async/await for data fetching
- Can access databases and backend services directly
- Cannot use React hooks (useState, useEffect, etc.)
- Reduce client-side JavaScript bundle size
- Render to HTML on the server

**Example:**

```jsx
// web/components/BlogPost.jsx
export default async function BlogPost({ id }) {
  // Direct database access (pseudo-code)
  const post = await db.query('SELECT * FROM posts WHERE id = ?', [id]);

  return (
    <article>
      <h1>{post.title}</h1>
      <div>{post.content}</div>
    </article>
  );
}
```

**Use server components for:**
- Data fetching
- Displaying content
- Reading from databases or APIs
- Reducing JavaScript sent to the browser

### Client Components

Client components run in the browser and must be marked with the `'use client'` directive.

**Features:**
- Can use React hooks (useState, useEffect, etc.)
- Can handle user interactions (onClick, onChange, etc.)
- Cannot use async/await at component level
- Include their JavaScript in the browser bundle

**Example:**

```jsx
// web/components/LikeButton.jsx
'use client';

import { useState } from 'react';

export function LikeButton({ postId }) {
  const [likes, setLikes] = useState(0);
  const [liked, setLiked] = useState(false);

  const handleLike = () => {
    if (!liked) {
      setLikes(l => l + 1);
      setLiked(true);
    }
  };

  return (
    <button
      onClick={handleLike}
      style={{
        padding: '8px 16px',
        background: liked ? '#ff6b6b' : '#4a5568',
        color: 'white',
        border: 'none',
        borderRadius: '4px',
        cursor: 'pointer'
      }}
    >
      {likes} {liked ? '❤️' : '🤍'}
    </button>
  );
}
```

**Use client components for:**
- Interactive UI elements
- Event handlers (onClick, onChange, etc.)
- Browser APIs (localStorage, window, etc.)
- State management with hooks

### Mixing Server and Client Components

You can import client components into server components:

```jsx
// web/components/BlogPost.jsx (Server Component)
import { LikeButton } from './LikeButton';

export default async function BlogPost({ id }) {
  const post = await fetchPost(id);

  return (
    <article>
      <h1>{post.title}</h1>
      <div>{post.content}</div>
      <LikeButton postId={post.id} />
    </article>
  );
}
```

## RenderedHtml API Reference

The `RenderedHtml` type is used to return SSR content from route handlers.

### Creating RenderedHtml

```rust
use virust_runtime::RenderedHtml;
use serde_json::json;

// Without props
let html = RenderedHtml::new("HomePage");

// With props
let html = RenderedHtml::with_props(
    "BlogPost",
    json!({"id": "123", "title": "My Post"})
);
```

### Route Attributes

Use the `#[render_component]` attribute to specify which component to render:

```rust
use virust_macros::{get, render_component};
use virust_runtime::RenderedHtml;

#[get]
#[render_component("ComponentName")]
async fn handler() -> RenderedHtml {
    RenderedHtml::new("ComponentName")
}
```

### Path Parameters with SSR

You can combine path parameters with SSR:

```rust
// src/api/posts/[id]/route.rs
use virust_macros::{get, render_component};
use virust_runtime::RenderedHtml;

#[get]
#[render_component("BlogPost")]
async fn get_post(#[path] id: String) -> RenderedHtml {
    RenderedHtml::with_props(
        "BlogPost",
        serde_json::json!({"id": id})
    )
}
```

### HTTP Methods

SSR works with any HTTP method:

```rust
use virust_macros::{get, post, render_component};

#[get]
#[render_component("ViewPage")]
async fn view_page() -> RenderedHtml {
    RenderedHtml::new("ViewPage")
}

#[post]
#[render_component("SubmitPage")]
async fn submit_page() -> RenderedHtml {
    RenderedHtml::new("SubmitPage")
}
```

## TypeScript Support

Virust has full TypeScript support for both Rust handlers and React components.

### TypeScript Components

```tsx
// web/components/UserProfile.tsx
interface UserProfileProps {
  id: string;
  showEmail?: boolean;
}

export default async function UserProfile({ id, showEmail }: UserProfileProps) {
  const user: User = await fetchUser(id);

  return (
    <div className="profile">
      <h1>{user.name}</h1>
      {showEmail && <p>{user.email}</p>}
    </div>
  );
}
```

### Type Safety

The component name in `#[render_component("...")]` must match the actual component file name. Virust will:
1. Look for the component in `web/components/`
2. Support both `.jsx` and `.tsx` extensions
3. Provide clear error messages if components are not found

## Project Structure

A typical SSR project has the following structure:

```
my-app/
├── src/
│   └── api/              # Rust API routes
│       ├── route.rs      # → GET /
│       └── posts/
│           └── [id]/
│               └── route.rs  # → GET /posts/:id
├── web/
│   ├── components/       # React components
│   │   ├── HomePage.jsx
│   │   ├── BlogPost.jsx
│   │   └── LikeButton.jsx  # Client component
│   └── index.html        # Entry point (optional)
├── Cargo.toml
└── main.rs
```

### Component Organization

- **Server Components**: Place in `web/components/` without `'use client'`
- **Client Components**: Place in `web/components/` with `'use client'` directive
- **Shared Utilities**: Can be placed in `web/utils/` or `web/lib/`

## Advanced Usage

### Data Fetching Patterns

#### Server-Side Data Fetching

```jsx
// web/components/Dashboard.jsx
export default async function Dashboard() {
  // Fetch data directly on the server
  const stats = await Promise.all([
    fetch('http://localhost:3000/api/users/count').then(r => r.json()),
    fetch('http://localhost:3000/api/posts/count').then(r => r.json()),
  ]);

  return (
    <div>
      <h1>Dashboard</h1>
      <p>Users: {stats[0].count}</p>
      <p>Posts: {stats[1].count}</p>
    </div>
  );
}
```

#### Client-Side Data Fetching

```jsx
// web/components/UserList.jsx
'use client';

import { useState, useEffect } from 'react';

export function UserList() {
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('/api/users')
      .then(r => r.json())
      .then(data => {
        setUsers(data);
        setLoading(false);
      });
  }, []);

  if (loading) return <div>Loading...</div>;

  return (
    <ul>
      {users.map(user => (
        <li key={user.id}>{user.name}</li>
      ))}
    </ul>
  );
}
```

### Error Handling

Virust provides development-friendly error pages when SSR fails. The error page includes:
- Component name that failed to render
- Error message and stack trace
- Helpful debugging information

In production, you may want to customize error handling:

```rust
use virust_runtime::RenderedHtml;

#[get]
#[render_component("RiskyComponent")]
async fn risky_handler() -> RenderedHtml {
    match fetch_data().await {
        Ok(data) => RenderedHtml::with_props("RiskyComponent", json!(data)),
        Err(e) => {
            // Return a fallback component
            RenderedHtml::new("ErrorPage")
        }
    }
}
```

### Conditional Rendering

You can conditionally render different components:

```rust
use virust_macros::get;
use virust_runtime::RenderedHtml;

#[get]
async fn conditional_route() -> RenderedHtml {
    if some_condition() {
        RenderedHtml::new("AuthenticatedPage")
    } else {
        RenderedHtml::new("LoginPage")
    }
}
```

## Hydration

Hydration is the process of making server-rendered HTML interactive in the browser. Virust handles this automatically:

1. **Server Render**: Component renders to HTML on the server
2. **Props Serialization**: Component props are serialized and embedded in the HTML
3. **Client Hydration**: React attaches event listeners and makes the page interactive

The hydration script is automatically included at `/bun/client.js` and requires no configuration.

## Hot Module Replacement

When running `virust dev`:
- Changes to Rust files trigger automatic recompilation and restart
- Changes to component files (`.jsx`, `.tsx`) trigger automatic re-rendering
- No manual refresh needed - see changes instantly

### HMR for Components

When you modify a component file:
1. Virust detects the change
2. Invalidates the component cache
3. Re-renders the component on next request
4. Browser refreshes automatically

## Troubleshooting

### Bun not installed

**Problem:** `Error: Failed to initialize Bun`

**Solution:** Install Bun from https://bun.sh

```bash
curl -fsSL https://bun.sh/install | bash
```

### Component not found

**Problem:** `Error: Component 'MyComponent' not found`

**Solution:** Ensure that:
1. Component file exists in `web/components/`
2. File has `.jsx` or `.tsx` extension
3. Component is exported as `export default`
4. Component name in `#[render_component("...")]` matches the file name (case-sensitive)

Example:
- File: `web/components/HomePage.jsx`
- Export: `export default async function HomePage()`
- Route: `#[render_component("HomePage")]`

### HMR not working

**Problem:** Changes to components not appearing

**Solution:**
1. Ensure `virust dev` is running
2. Check that component file changes are being saved
3. Look for file watcher messages in the dev server output
4. Try manually refreshing the browser

### Async/await errors

**Problem:** Syntax errors when using async/await in components

**Solution:** Remember that async/await is only allowed in server components:

```jsx
// ✅ Correct - Server component
export default async function BlogPost() {
  const post = await fetchPost(id);
  return <div>{post.title}</div>;
}

// ❌ Wrong - Client component with async
'use client';
export default async function BlogPost() {  // Error!
  const post = await fetchPost(id);
  return <div>{post.title}</div>;
}
```

### Props not passing correctly

**Problem:** Component receives undefined props

**Solution:** Ensure props are properly serialized:

```rust
// Rust
use serde_json::json;

#[get]
#[render_component("BlogPost")]
async fn get_post(#[path] id: String) -> RenderedHtml {
    RenderedHtml::with_props(
        "BlogPost",
        json!({  // Note the double braces
            "id": id,
            "title": "My Post"
        })
    )
}
```

And in TypeScript:

```tsx
interface BlogPostProps {
  id: string;
  title: string;
}

export default async function BlogPost({ id, title }: BlogPostProps) {
  return <div>{title}</div>;
}
```

### Port already in use

**Problem:** `Error: Address already in use (os error 98)`

**Solution:** Either:
1. Stop the existing process using port 3000
2. Use a different port: `virust dev --port 3001`

### Build errors

**Problem:** Compilation errors when building for production

**Solution:**
1. Ensure all dependencies are in `Cargo.toml`
2. Check for version conflicts
3. Run `cargo clean` and try again
4. Verify VIRUST_PATH is set correctly if using path dependencies

## Best Practices

### Performance

1. **Use Server Components by Default**: They reduce client-side JavaScript
2. **Lazy Load Client Components**: Only load interactive components when needed
3. **Optimize Data Fetching**: Use Promise.all for parallel requests
4. **Cache When Possible**: Implement caching for expensive operations

```jsx
// Good - Parallel data fetching
export default async function Dashboard() {
  const [users, posts, comments] = await Promise.all([
    fetch('/api/users').then(r => r.json()),
    fetch('/api/posts').then(r => r.json()),
    fetch('/api/comments').then(r => r.json()),
  ]);

  return <DashboardContent users={users} posts={posts} comments={comments} />;
}
```

### Code Organization

1. **Separate Concerns**: Keep server and client components in separate files when possible
2. **Use TypeScript**: Provides better type safety and developer experience
3. **Component Reusability**: Design components to be reusable with clear props interfaces
4. **File Naming**: Use PascalCase for component files (e.g., `UserProfile.jsx`)

### Error Handling

1. **Graceful Degradation**: Provide fallbacks for failed data fetching
2. **Error Boundaries**: Use React error boundaries for client components
3. **Development Errors**: Leverage Virust's detailed error pages during development

```jsx
'use client';

import { Component } from 'react';

export class ErrorBoundary extends Component {
  state = { hasError: false };

  static getDerivedStateFromError(error) {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return <h1>Something went wrong.</h1>;
    }
    return this.props.children;
  }
}
```

## Next Steps

- Explore the `ssr-blog` template for a complete working example
- Check the `ssr-dashboard` template for admin dashboard patterns
- Read the [v0.4 Release Notes](/docs/plans/2026-03-06-virust-v0.4-release.md)
- Review the [Main README](/README.md) for general Virust features

## Additional Resources

- [React Documentation](https://react.dev/)
- [Bun Documentation](https://bun.sh/docs)
- [Next.js Documentation](https://nextjs.org/docs) (for similar patterns)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
