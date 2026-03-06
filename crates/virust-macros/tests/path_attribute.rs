use virust_macros::get;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
}

#[get]
async fn get_user(#[path] id: String) -> String {
    let response = UserResponse {
        id: id.clone(),
        name: format!("User {}", id),
    };
    serde_json::to_string(&response).unwrap()
}

fn main() {}
