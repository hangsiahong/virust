use virust_macros::{get, render_component};
use virust_runtime::RenderedHtml;
use serde_json::json;

/// Dashboard data structure
#[derive(Clone, Debug)]
pub struct MetricData {
    pub label: String,
    pub value: f64,
    pub change: f64,
    pub trend: String, // "up" or "down"
}

/// Chart data point
#[derive(Clone, Debug)]
pub struct ChartPoint {
    pub label: String,
    pub value: f64,
}

/// Get dashboard metrics
fn get_dashboard_metrics() -> Vec<MetricData> {
    vec![
        MetricData {
            label: "Total Users".to_string(),
            value: 1250.0,
            change: 12.5,
            trend: "up".to_string(),
        },
        MetricData {
            label: "Active Users".to_string(),
            value: 342.0,
            change: 8.2,
            trend: "up".to_string(),
        },
        MetricData {
            label: "Revenue".to_string(),
            value: 45320.0,
            change: 23.1,
            trend: "up".to_string(),
        },
        MetricData {
            label: "Conversion Rate".to_string(),
            value: 3.2,
            change: -0.5,
            trend: "down".to_string(),
        },
        MetricData {
            label: "Page Views".to_string(),
            value: 89500.0,
            change: 15.3,
            trend: "up".to_string(),
        },
        MetricData {
            label: "Avg. Session Duration".to_string(),
            value: 4.5,
            change: 5.8,
            trend: "up".to_string(),
        },
    ]
}

/// Get chart data for the last 7 days
fn get_chart_data() -> Vec<ChartPoint> {
    vec![
        ChartPoint {
            label: "Mon".to_string(),
            value: 1200.0,
        },
        ChartPoint {
            label: "Tue".to_string(),
            value: 1450.0,
        },
        ChartPoint {
            label: "Wed".to_string(),
            value: 1320.0,
        },
        ChartPoint {
            label: "Thu".to_string(),
            value: 1680.0,
        },
        ChartPoint {
            label: "Fri".to_string(),
            value: 1890.0,
        },
        ChartPoint {
            label: "Sat".to_string(),
            value: 2100.0,
        },
        ChartPoint {
            label: "Sun".to_string(),
            value: 1950.0,
        },
    ]
}

/// Get recent activities
fn get_recent_activities() -> Vec<serde_json::Value> {
    vec![
        json!({
            "id": "1",
            "type": "user",
            "message": "New user registered: john.doe@example.com",
            "time": "2 minutes ago",
            "icon": "👤"
        }),
        json!({
            "id": "2",
            "type": "purchase",
            "message": "Purchase completed: $99.00",
            "time": "5 minutes ago",
            "icon": "💳"
        }),
        json!({
            "id": "3",
            "type": "alert",
            "message": "Server load increased to 75%",
            "time": "10 minutes ago",
            "icon": "⚠️"
        }),
        json!({
            "id": "4",
            "type": "success",
            "message": "Deployment successful: v2.4.1",
            "time": "15 minutes ago",
            "icon": "✅"
        }),
        json!({
            "id": "5",
            "type": "user",
            "message": "User upgraded to premium plan",
            "time": "20 minutes ago",
            "icon": "⭐"
        }),
    ]
}

/// Dashboard page with server-side rendering
#[get]
#[render_component("Dashboard")]
pub async fn dashboard() -> RenderedHtml {
    let metrics = get_dashboard_metrics();
    let chart_data = get_chart_data();
    let activities = get_recent_activities();

    let metrics_json: Vec<serde_json::Value> = metrics.iter().map(|m| {
        json!({
            "label": m.label,
            "value": m.value,
            "change": m.change,
            "trend": m.trend,
        })
    }).collect();

    let chart_json: Vec<serde_json::Value> = chart_data.iter().map(|p| {
        json!({
            "label": p.label,
            "value": p.value,
        })
    }).collect();

    let data = json!({
        "title": "Analytics Dashboard",
        "metrics": metrics_json,
        "chartData": chart_json,
        "activities": activities,
        "lastUpdated": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    });

    RenderedHtml::with_props("Dashboard", data)
}
