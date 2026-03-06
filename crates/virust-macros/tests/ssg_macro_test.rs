use virust_macros::{get, ssg};

// Test that #[ssg] with revalidate compiles and works
#[get]
#[ssg(revalidate = 3600)]
pub async fn test_route_with_revalidate() -> &'static str {
    "test"
}

// Test that #[ssg] without parameters compiles and works
#[get]
#[ssg]
pub async fn test_route_static() -> &'static str {
    "static"
}

// Test that routes are still callable
#[test]
fn test_ssg_routes_work() {
    // The simplified macro should pass through functions unchanged
    // so they should still be callable
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(test_route_with_revalidate());
    assert_eq!(result, "test");

    let result2 = rt.block_on(test_route_static());
    assert_eq!(result2, "static");
}

// Test that macro can be used with different revalidate times
#[get]
#[ssg(revalidate = 60)]
pub async fn test_isr_route() -> &'static str {
    "isr"
}

#[get]
#[ssg(revalidate = 300)]
pub async fn test_isr_five_min() -> &'static str {
    "five_min"
}

#[test]
fn test_isr_routes_work() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(test_isr_route());
    assert_eq!(result, "isr");

    let result2 = rt.block_on(test_isr_five_min());
    assert_eq!(result2, "five_min");
}

// Test that macro works with different return types
#[get]
#[ssg]
pub async fn test_string_return() -> String {
    "string".to_string()
}

#[get]
#[ssg(revalidate = 120)]
pub async fn test_html_return() -> String {
    "<html>test</html>".to_string()
}

#[test]
fn test_different_return_types() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(test_string_return());
    assert_eq!(result, "string");

    let result2 = rt.block_on(test_html_return());
    assert_eq!(result2, "<html>test</html>");
}

// Test that macro preserves function attributes
#[get]
#[ssg(revalidate = 600)]
#[allow(dead_code)]
pub async fn test_with_attributes() -> &'static str {
    "attributes"
}

#[test]
fn test_macro_preserves_attributes() {
    // The macro should preserve other attributes like #[allow(dead_code)]
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(test_with_attributes());
    assert_eq!(result, "attributes");
}
