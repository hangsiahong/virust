use virust_macros::{get, post, put, delete};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateRequest {
    pub name: String,
    pub value: i32,
}

// Test with both path and body parameters
#[post]
async fn update_item(#[path] id: String, #[body] data: UpdateRequest) -> String {
    format!("Updating item {} with {:?}", id, data)
}

#[put]
async fn replace_item(#[path] id: String, #[body] data: UpdateRequest) -> String {
    format!("Replacing item {} with {:?}", id, data)
}

#[delete]
async fn delete_item(#[path] id: String, #[body] reason: String) -> String {
    format!("Deleting item {} because {}", id, reason)
}

#[get]
async fn get_item(#[path] id: String, #[body] filter: String) -> String {
    format!("Getting item {} with filter {}", id, filter)
}

#[test]
fn test_path_and_body_params() {
    // Test that macros compile with both path and body parameters
    assert!(true);
}

fn main() {}
