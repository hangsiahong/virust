use axum::{response::Json, routing::get, Router};
use serde_json::json;

pub fn create_http_router() -> Router {
    Router::new().route("/", get(|| async { Json(json!({"status": "ok"})) }))
}