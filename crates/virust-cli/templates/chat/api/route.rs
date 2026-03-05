use virust_macros::ws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use virust_protocol::InMemoryPersistence;

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub username: String,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ChatEntry {
    pub id: String,
    pub username: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub ok: bool,
    pub message: Option<String>,
}

// Thread-safe message history
lazy_static::lazy_static! {
    static ref MESSAGE_HISTORY: Arc<RwLock<Vec<ChatEntry>>> = Arc::new(RwLock::new(Vec::new()));
}

#[ws]
async fn route(msg: ChatMessage) -> ChatResponse {
    let entry = ChatEntry {
        id: uuid::Uuid::new_v4().to_string(),
        username: msg.username.clone(),
        message: msg.message.clone(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Store message in history
    let mut history = MESSAGE_HISTORY.write().await;
    history.push(entry.clone());

    // Keep only last 100 messages
    if history.len() > 100 {
        history.remove(0);
    }

    println!("[{}] {}: {}", entry.id, msg.username, msg.message);
    ChatResponse {
        ok: true,
        message: Some("Message received".to_string()),
    }
}

/// Get message history endpoint
pub async fn history() -> String {
    let history = MESSAGE_HISTORY.read().await;
    serde_json::to_string(&*history).unwrap_or_else(|_| "[]".to_string())
}
