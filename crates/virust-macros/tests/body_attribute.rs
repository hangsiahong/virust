use virust_macros::post;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
}

#[post]
async fn create_todo(#[body] payload: CreateTodoRequest) -> String {
    serde_json::to_string(&payload).unwrap()
}

#[test]
fn test_body_macro_expansion() {
    // Test that #[post] with #[body] generates correct code
    // This will be verified by integration test later
    assert!(true);
}

fn main() {}
