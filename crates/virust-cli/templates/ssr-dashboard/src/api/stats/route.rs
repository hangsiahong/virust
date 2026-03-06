use virust_macros::{get, ssg, cache};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_users: i64,
    pub active_sessions: i64,
    pub revenue: f64,
    pub conversion_rate: f64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub values: Vec<i64>,
}

// Simulated stats storage
lazy_static::lazy_static! {
    static ref STATS: Arc<RwLock<Stats>> = Arc::new(RwLock::new(
        Stats {
            total_users: 12500,
            active_sessions: 342,
            revenue: 45230.50,
            conversion_rate: 3.2,
            updated_at: chrono::Utc::now().timestamp(),
        }
    ));
}

/// Get dashboard stats (SSG + cached for 5 minutes)
#[get]
#[ssg]
#[cache(ttl = 300)]
async fn route() -> String {
    let stats = STATS.read().await;
    serde_json::to_string(&*stats).unwrap_or_else(|_| "{}".to_string())
}

/// Get chart data (cached for 10 minutes)
#[get]
#[cache(ttl = 600)]
async fn route() -> String {
    // Generate simulated chart data
    let labels: Vec<String> = (0..7).map(|i| {
        let date = chrono::Utc::now() - chrono::Duration::days(6 - i);
        date.format("%Y-%m-%d").to_string()
    }).collect();

    let values: Vec<i64> = (0..7).map(|_| {
        rand::random::<i64>() % 1000 + 500
    }).collect();

    let data = ChartData { labels, values };
    serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())
}
