use virust_macros::{get, post, cache};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub title: String;
    pub description: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: i64,
}

lazy_static::lazy_static! {
    static ref TODOS: Arc<RwLock<Vec<Todo>>> = Arc::new(RwLock::new(Vec::new()));
}

/// Get all todos (cached for 1 minute)
#[get]
#[cache(ttl = 60)]
async fn route() -> String {
    let todos = TODOS.read().await;
    serde_json::to_string(&*todos).unwrap_or_else(|_| "[]".to_string())
}

/// Create a new todo
#[post]
async fn route(#[body] payload: CreateTodoRequest) -> String {
    let todo = Todo {
        id: uuid::Uuid::new_v4().to_string(),
        title: payload.title,
        description: payload.description,
        completed: false,
        created_at: chrono::Utc::now().timestamp(),
    };

    let mut todos = TODOS.write().await;
    todos.push(todo.clone());

    serde_json::to_string(&todo).unwrap_or_else(|_| "{}".to_string())
}
