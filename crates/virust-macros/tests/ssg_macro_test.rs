use virust_macros::ssg;
use virust_build::SsgRouteMetadata;

#[ssg(revalidate = 60)]
async fn test_route() -> String {
    "test".to_string()
}

#[ssg]
async fn another_route() -> String {
    "another".to_string()
}

#[test]
fn test_macro_expansion() {
    // Test that macro expands without errors
    // The macro should generate the metadata structs
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(test_route());
    assert_eq!(result, "test");

    let result2 = rt.block_on(another_route());
    assert_eq!(result2, "another");
}

#[test]
fn test_metadata() {
    // Test that the metadata is correctly set
    assert_eq!(test_routeSsgMeta::REVALIDATE, Some(60));
    assert_eq!(test_routeSsgMeta::route_path(), "test_route");

    assert_eq!(another_routeSsgMeta::REVALIDATE, None);
    assert_eq!(another_routeSsgMeta::route_path(), "another_route");
}

