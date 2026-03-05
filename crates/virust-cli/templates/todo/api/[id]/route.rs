use virust_macros::{get, put, delete};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use virust_protocol::{InMemoryPersistence, Persistence};

#[derive(Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: i64,
}

// Thread-safe persistence (shared with todos/route.rs)
lazy_static::lazy_static! {
    static ref PERSISTENCE: Arc<InMemoryPersistence> = Arc::new(InMemoryPersistence::new());
}

/// Get a specific todo by ID
#[get]
async fn route(#[path] id: String) -> String {
    match PERSISTENCE.get("todos", &id).await {
        Ok(Some(todo)) => serde_json::to_string(&todo).unwrap_or_else(|_| "{}".to_string()),
        Ok(None) => serde_json::json!({"error": "Todo not found"}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Update a todo by ID
#[put]
async fn route(#[path] id: String, #[body] payload: UpdateTodoRequest) -> String {
    // First get the existing todo
    let existing = match PERSISTENCE.get("todos", &id).await {
        Ok(Some(todo)) => todo,
        Ok(None) => return serde_json::json!({"error": "Todo not found"}).to_string(),
        Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
    };

    // Update fields if provided
    let mut updated = existing;
    if let Some(title) = payload.title {
        updated["title"] = serde_json::Value::String(title);
    }
    if let Some(description) = payload.description {
        updated["description"] = serde_json::Value::String(description);
    }
    if let Some(completed) = payload.completed {
        updated["completed"] = serde_json::Value::Bool(completed);
    }

    match PERSISTENCE.update("todos", &id, updated.clone()).await {
        Ok(_) => serde_json::to_string(&updated).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Delete a todo by ID
#[delete]
async fn route(#[path] id: String) -> String {
    match PERSISTENCE.delete("todos", &id).await {
        Ok(_) => serde_json::json!({"success": true}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
