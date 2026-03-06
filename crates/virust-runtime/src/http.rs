use axum::{response::{IntoResponse, Response, Json}, routing::get, Router};
use serde_json::json;
use axum::http::StatusCode;

async fn root_handler() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

/// Handler for the /__types endpoint
pub async fn types_handler() -> impl IntoResponse {
    // Get all registered type definitions
    let type_definitions = crate::get_registered_types();

    // Generate TypeScript code
    let generator = crate::TypeScriptGenerator::new(type_definitions);
    let ts_code = generator.generate();

    // Return as plain text with proper headers
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/typescript; charset=utf-8")
        .header("Cache-Control", "no-cache")
        .body(ts_code)
        .unwrap()
}

pub fn create_http_router() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/__types", get(types_handler))
}