use virust_macros::get;

#[get]
pub async fn test_handler() -> &'static str {
    "test"
}
