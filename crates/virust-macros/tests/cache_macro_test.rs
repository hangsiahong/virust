use virust_macros::cache;
use virust_build::CacheRouteMetadata;

#[cache(max_age = 60)]
async fn test_cache_route() -> String {
    "cached_data".to_string()
}

#[cache(max_age = 300)]
async fn another_cache_route() -> String {
    "more_cached_data".to_string()
}

#[cache(max_age = 3600)]
async fn long_cache_route() -> String {
    "long_cached".to_string()
}

#[test]
fn test_macro_expansion() {
    // Test that macro expands without errors
    // The macro should generate the metadata structs
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(test_cache_route());
    assert_eq!(result, "cached_data");

    let result2 = rt.block_on(another_cache_route());
    assert_eq!(result2, "more_cached_data");

    let result3 = rt.block_on(long_cache_route());
    assert_eq!(result3, "long_cached");
}

#[test]
fn test_metadata() {
    // Test that the metadata is correctly set
    assert_eq!(test_cache_routeCacheMeta::MAX_AGE, 60);
    assert_eq!(test_cache_routeCacheMeta::route_path(), "test_cache_route");

    assert_eq!(another_cache_routeCacheMeta::MAX_AGE, 300);
    assert_eq!(another_cache_routeCacheMeta::route_path(), "another_cache_route");

    assert_eq!(long_cache_routeCacheMeta::MAX_AGE, 3600);
    assert_eq!(long_cache_routeCacheMeta::route_path(), "long_cache_route");
}
