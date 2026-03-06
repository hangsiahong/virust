use virust_macros::{get, cache};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub active_sessions: i64,
    pub revenue: f64,
}

lazy_static::lazy_static! {
    static ref STATS: Arc<RwLock<DashboardStats>> = Arc::new(RwLock::new(
        DashboardStats {
            total_users: 1000,
            active_sessions: 150,
            revenue: 25000.0,
        }
    ));
}

/// Get dashboard stats (SSR + cached for 5 minutes)
#[get]
#[cache(ttl = 300)]
async fn route() -> String {
    let stats = STATS.read().await;
    serde_json::to_string(&*stats).unwrap_or_else(|_| "{}".to_string())
}
