use virust_macros::get;

#[get]
async fn get_user(#[path] id: String, #[path] name: String) -> String {
    format!("User {}: {}", id, name)
}

#[test]
fn test_multi_path() {
    assert!(true);
}

fn main() {}
