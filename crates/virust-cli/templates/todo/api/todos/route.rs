use virust_macros::{get, post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use virust_protocol::{InMemoryPersistence, Persistence};

#[derive(Deserialize, Serialize)]
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
    pub created_at: i64,
}

// Thread-safe persistence
lazy_static::lazy_static! {
    static ref PERSISTENCE: Arc<InMemoryPersistence> = Arc::new(InMemoryPersistence::new());
}

/// List all todos
#[get]
async fn route() -> String {
    match PERSISTENCE.list("todos").await {
        Ok(todos) => serde_json::to_string(&todos).unwrap_or_else(|_| "[]".to_string()),
        Err(_) => "[]".to_string(),
    }
}

/// Create a new todo
#[post]
async fn route(#[body] payload: CreateTodoRequest) -> String {
    let todo = serde_json::json!({
        "title": payload.title,
        "description": payload.description,
        "completed": false,
        "created_at": chrono::Utc::now().timestamp(),
    });

    match PERSISTENCE.create("todos", todo).await {
        Ok(id) => {
            let response = serde_json::json!({
                "id": id,
                "title": payload.title,
                "description": payload.description,
                "completed": false,
                "created_at": chrono::Utc::now().timestamp(),
            });
            serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
        }
        Err(e) => {
            serde_json::json!({"error": e.to_string()}).to_string()
        }
    }
}
