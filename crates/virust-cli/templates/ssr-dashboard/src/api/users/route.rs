use virust_macros::{get, cache};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: i64,
}

// Simulated user storage
lazy_static::lazy_static! {
    static ref USERS: Arc<RwLock<Vec<User>>> = Arc::new(RwLock::new(
        vec![
            User {
                id: "1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                role: "Admin".to_string(),
                created_at: 1704067200,
            },
            User {
                id: "2".to_string(),
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
                role: "User".to_string(),
                created_at: 1704153600,
            },
            User {
                id: "3".to_string(),
                name: "Bob Johnson".to_string(),
                email: "bob@example.com".to_string(),
                role: "User".to_string(),
                created_at: 1704240000,
            },
        ]
    ));
}

/// Get all users (cached for 5 minutes)
#[get]
#[cache(ttl = 300)]
async fn route() -> String {
    let users = USERS.read().await;
    serde_json::to_string(&*users).unwrap_or_else(|_| "[]".to_string())
}
