use virust_macros::get;

#[get]
async fn route() -> String {
    r#"{"message": "Hello from {{project_name}}!"}"#.to_string()
}
