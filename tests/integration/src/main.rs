use virust_macros::{get, ws};
use virust_runtime::VirustApp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct PingRequest {
    msg: String,
}

#[derive(Serialize)]
struct PingResponse {
    reply: String,
}

#[ws]
async fn ping(req: PingRequest) -> PingResponse {
    PingResponse {
        reply: format!("pong: {}", req.msg),
    }
}

#[get]
async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    let app = VirustApp::new();
    let router = app.router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Test server listening on http://127.0.0.1:3000");
    axum::serve(listener, router).await.unwrap();
}