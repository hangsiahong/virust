# Virust

A **real-time Rust backend framework** optimized for AI-assisted development.

## Features

- **WebSocket-first** - Real-time bidirectional messaging with JSON-RPC
- **Filesystem routing** - Next.js style: `api/chat/route.rs` → `/api/chat`
- **Macro-powered** - `#[ws]`, `#[get]`, `#[post]`, `#[put]`, `#[delete]`
- **Type-safe** - Full TypeScript code generation from Rust handlers
- **Minimal boilerplate** - Write only business logic

## Quick Start

### Installation

```bash
cargo install virust
```

### Create a Project

```bash
virust init my-app
cd my-app
```

### Development

```bash
virust dev
```

### Production Build

```bash
virust build --release
```

## Example

```rust
// api/chat/route.rs
use virust_macros::{ws, post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub message: String,
}

#[ws]
async fn chat(msg: ChatMessage) -> ChatResponse {
    ChatResponse { ok: true }
}

#[post]
async fn send_message(#[body] msg: ChatMessage) -> Message {
    // Save to database, etc.
    Message::new(msg.message)
}
```

## Architecture

```
virust/
├ crates/
│ ├ virust-protocol    # Shared types (RPC, HTTP, errors)
│ ├ virust-macros      # Proc macros + TS generation
│ ├ virust-runtime     # WebSocket + HTTP servers
│ └ virust-cli         # Project scaffolding (init, dev, build)
```

## License

MIT