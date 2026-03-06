use virust_macros::get;

#[get]
async fn get_user(#[path] id: String) -> i32 {
    42
}

#[test]
fn test_different_return() {
    assert!(true);
}

fn main() {}
