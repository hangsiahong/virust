# Virust

A **real-time Rust backend framework** optimized for AI-assisted development.

## Features

- **Server-Side Rendering** - Next.js-style SSR with Bun for React components
- **React Server Components** - Async components with server-side data fetching
- **Client Components** - Interactive components with React hooks
- **WebSocket-first** - Real-time bidirectional messaging with JSON-RPC
- **Filesystem routing** - Next.js style: `api/chat/route.rs` → `/api/chat`
- **Macro-powered** - `#[ws]`, `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[render_component]`
- **Type-safe** - Full TypeScript support for Rust and JavaScript
- **Parameter extraction** - `#[path]` and `#[body]` attributes for clean syntax
- **Auto-discovery** - Routes automatically discovered from `api/` directory
- **Single-port dev** - Everything on `:3000` with hot module replacement
- **Minimal boilerplate** - Write only business logic

## Quick Start

### Installation

```bash
cargo install virust
```

### Create a Project

```bash
# Basic API project
virust init my-app

# SSR blog application
virust init my-blog -t ssr-blog

# SSR dashboard application
virust init my-dashboard -t ssr-dashboard
cd my-app
```

### Development

```bash
virust dev
```

This starts a single server on port 3000 that serves:
- Static files from `web/`
- API routes from `api/`
- Server-rendered React components (SSR projects)
- WebSocket connections
- TypeScript types at `/api/__types`

### Production Build

```bash
virust build --release
```

## Examples

### WebSocket Handler

```rust
// api/chat/route.rs
use virust_macros::ws;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub ok: bool,
    pub message: String,
}

#[ws]
async fn chat(msg: ChatMessage) -> ChatResponse {
    ChatResponse {
        ok: true,
        message: format!("{}: {}", msg.username, msg.message),
    }
}
```

### HTTP Handlers with Parameter Extraction

```rust
// api/todos/route.rs
use virust_macros::{get, post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

#[post]
async fn create_todo(#[body] payload: CreateTodoRequest) -> TodoResponse {
    // payload is automatically deserialized from JSON
    TodoResponse {
        id: uuid::Uuid::new_v4().to_string(),
        title: payload.title,
        description: payload.description,
        completed: false,
    }
}

#[get]
async fn get_todos() -> Vec<TodoResponse> {
    // Return all todos
    vec![]
}
```

### Path Parameters

```rust
// api/todos/[id]/route.rs
use virust_macros::{get, put, delete};
use serde::Serialize;

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
}

#[get]
async fn get_todo(#[path] id: String) -> TodoResponse {
    // id is extracted from URL path /api/todos/:id
    TodoResponse {
        id: id.clone(),
        title: format!("Todo {}", id),
    }
}

#[put]
async fn update_todo(#[path] id: String, #[body] data: UpdateTodoRequest) -> TodoResponse {
    // Both path and body parameters work together
    TodoResponse {
        id,
        title: data.title,
    }
}

#[delete]
async fn delete_todo(#[path] id: String) -> String {
    format!("Deleted todo {}", id)
}
```

### Server-Side Rendering (v0.4)

```rust
// api/route.rs
use virust_macros::{get, render_component};
use virust_runtime::RenderedHtml;

#[get]
#[render_component("HomePage")]
async fn index() -> RenderedHtml {
    RenderedHtml::new("HomePage")
}
```

```jsx
// web/components/HomePage.jsx
export default async function HomePage() {
  const posts = await fetch('/api/posts').then(r => r.json());

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

See the [SSR Guide](docs/ssr-guide.md) for complete documentation on server-side rendering.

## TypeScript Generation

Virust automatically generates TypeScript interfaces for all your route handlers. Access them at:

```
http://localhost:3000/api/__types
```

Example generated TypeScript:

```typescript
export interface CreateTodoRequest {
  title: string;
  description?: string | null;
}

export interface TodoResponse {
  id: string;
  title: string;
  description?: string | null;
  completed: boolean;
}

export function createTodo(payload: CreateTodoRequest): Promise<TodoResponse>;
export function getTodos(): Promise<TodoResponse[]>;
export function getTodo(id: string): Promise<TodoResponse>;
```

## Filesystem Routing

Routes are automatically discovered from the `api/` directory:

```
api/
├ chat/
│ └ route.rs          → POST /api/chat
├ todos/
│ ├ route.rs          → GET /api/todos, POST /api/todos
│ └ [id]/
│   └ route.rs        → GET /api/todos/:id, PUT /api/todos/:id, DELETE /api/todos/:id
```

Dynamic routes use `[param]` syntax (similar to Next.js):

- `api/users/[id]/route.rs` → `/api/users/:id`
- `api/posts/[slug]/comments/[comment_id]/route.rs` → `/api/posts/:slug/comments/:comment_id`

## Hot Module Replacement

When running `virust dev`:
- Frontend changes in `web/` trigger automatic page reload
- Backend changes in `api/` trigger automatic recompilation and restart
- No manual refresh needed - see changes instantly

## Architecture

```
virust/
├ crates/
│ ├ virust-protocol     # Shared types (RPC, HTTP, errors)
│ ├ virust-macros       # Proc macros + TS generation
│ ├ virust-runtime      # WebSocket + HTTP servers + SSR
│ ├ virust-cli          # Project scaffolding (init, dev, build)
│ ├ virust-typescript   # TypeScript code generation
│ └ virust-bun          # Bun integration for SSR
```

## v0.4 Features

The latest release includes:

- **Server-Side Rendering**: Next.js-style SSR with Bun runtime
- **React Server Components**: Async components with direct data fetching
- **Client Components**: Interactive components with React hooks
- **TypeScript Support**: Full TS/TSX support for components
- **Hydration**: Automatic client-side hydration for interactivity
- **Component HMR**: Hot module replacement for component changes
- **Error Pages**: Development-friendly error pages for failed renders

See [v0.4 Release Notes](docs/plans/2026-03-06-virust-v0.4-release.md) for details.

## Documentation

- [SSR Guide](docs/ssr-guide.md) - Complete guide to server-side rendering
- [v0.4 Release Notes](docs/plans/2026-03-06-virust-v0.4-release.md) - Latest release notes
- [v0.3 Release Notes](docs/plans/2026-03-06-virust-v0.3-release.md) - Previous release notes

## v0.3 Features

Previous v0.3 features include:

- **Parameter Extraction**: Use `#[path]` for URL parameters and `#[body]` for JSON bodies
- **Complete TypeScript Generation**: Full type definitions with all struct fields
- **Automatic Route Discovery**: Scan `api/` directory and register routes automatically
- **Single-Port Development**: Everything runs on `:3000` with HMR for instant feedback
- **Type-Safe Routing**: Compile-time guarantees for route signatures

## License

MIT