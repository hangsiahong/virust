use virust_macros::{get, post, put, delete};

// Test i32 return type
#[get]
async fn get_number(#[path] id: String) -> i32 {
    42
}

// Test String return type
#[post]
async fn post_string(#[path] id: String) -> String {
    "hello".to_string()
}

// Test bool return type
#[put]
async fn put_bool(#[path] id: String) -> bool {
    true
}

// Test Vec return type
#[delete]
async fn delete_vec(#[path] id: String) -> Vec<i32> {
    vec![1, 2, 3]
}

// Test Option return type
#[get]
async fn get_option(#[path] id: String) -> Option<String> {
    Some("value".to_string())
}

// Test Result return type
#[post]
async fn post_result(#[path] id: String) -> Result<String, String> {
    Ok("success".to_string())
}

#[test]
fn test_various_return_types() {
    // This test verifies compilation succeeds for various return types
    assert!(true);
}

fn main() {}
